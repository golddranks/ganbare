#![recursion_limit = "1024"]
#![feature(proc_macro)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros, binary_macros)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate mime;

extern crate dotenv;
extern crate crypto;
extern crate chrono;
extern crate rand;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate pencil;



use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;

pub use diesel::pg::PgConnection;

pub mod schema;
pub mod models;
use models::*;
pub mod password;
pub mod errors {

    error_chain! {
        foreign_links {
            ::std::num::ParseIntError, ParseIntError;
            ::std::io::Error, StdIoError;
            ::diesel::result::Error, DieselError;
            ::pencil::PencilError, PencilError;
        }
        errors {
            NoSuchUser(email: String) {
                description("No such user exists")
                display("No user with e-mail address {} exists.", email)
            }
            EmailAddressTooLong {
                description("E-mail address too long")
                display("A valid e-mail address can be 254 characters at maximum.")
            }
            EmailAddressNotValid {
                description("E-mail address not valid")
                display("An e-mail address must contain the character '@'.")
            }
            PasswordTooShort {
                description("Password too short")
                display("A valid password must be at least 8 characters (bytes).")
            }
            PasswordTooLong {
                description("Password too long")
                display("A valid password must be at maximum 1024 characters (bytes).")
            }
            PasswordDoesntMatch {
                description("Password doesn't match")
                display("Password doesn't match.")
            }
            AuthError {
                description("Can't authenticate user")
                display("Username (= e-mail) or password doesn't match.")
            }
            BadSessId {
                description("Malformed session ID!")
                display("Malformed session ID!")
            }
            NoSuchSess {
                description("Session doesn't exist!")
                display("Session doesn't exist!")
            }
            FormParseError {
                description("Can't parse the HTTP form!")
                display("Can't parse the HTTP form!")
            }
        }
    }
}


use errors::*;

pub fn db_connect() -> Result<PgConnection> {
    dotenv().ok();
    let database_url = env::var("GANBARE_DATABASE_URL")
        .chain_err(|| "GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")?;
    PgConnection::establish(&database_url)
        .chain_err(|| "Error connecting to database!")
}

pub fn get_user_by_email(conn : &PgConnection, user_email : &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    users
        .filter(email.eq(user_email))
        .first(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(user_email.into())),
                e => e.caused_err(|| "Error when trying to retrieve user!"),
        })
}

fn get_user_pass_by_email(conn : &PgConnection, user_email : &str) -> Result<(User, Password)> {
    use schema::users;
    use schema::passwords;
    use diesel::result::Error::NotFound;

    users::table
        .inner_join(passwords::table)
        .filter(users::email.eq(user_email))
        .first(&*conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(user_email.into())),
                e => e.caused_err(|| "Error when trying to retrieve user!"),
        })
}

pub fn add_user(conn : &PgConnection, email : &str, password : &str) -> Result<User> {
    use schema::{users, passwords};

    if email.len() > 254 { return Err(ErrorKind::EmailAddressTooLong.into()) };
    if !email.contains("@") { return Err(ErrorKind::EmailAddressNotValid.into()) };

    let pw = password::set_password(password).chain_err(|| "Setting password didn't succeed!")?;

    let new_user = NewUser {
        email : email,
    };

    let user : User = diesel::insert(&new_user)
        .into(users::table)
        .get_result(conn)
        .chain_err(|| "Couldn't create a new user!")?;

    diesel::insert(&pw.into_db(user.id))
        .into(passwords::table)
        .execute(conn)
        .chain_err(|| "Couldn't insert the new password into database!")?;

    Ok(user)
}

pub fn remove_user(conn : &PgConnection, rm_email : &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    diesel::delete(users.filter(email.eq(rm_email)))
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(rm_email.into())),
                e => e.caused_err(|| "Couldn't remove the user!"),
        })
}

pub fn auth_user(conn : &PgConnection, email : &str, plaintext_pw : &str) -> Result<User> {
    let (user, hashed_pw_from_db) = get_user_pass_by_email(conn, email)
                                .map_err(|err| {
                                    if let &ErrorKind::NoSuchUser(_) = err.kind() {
                                        return ErrorKind::AuthError.to_err();
                                    };
                                    err
                                })?;
    let _ = password::check_password(plaintext_pw, hashed_pw_from_db.into())
                                .map_err(|err| {
                                    if let &ErrorKind::PasswordDoesntMatch = err.kind() {
                                        return ErrorKind::AuthError.to_err();
                                    };
                                    err
                                })?;
    Ok(user)
}

pub const SESSID_BITS : usize = 128;

/// TODO refactor this function, this is only a temporary helper
pub fn sess_to_hex(sess : &Session) -> String {
    use data_encoding::base16;
    base16::encode(sess.sess_id.as_ref())
}

/// TODO refactor this function, this is only a temporary helper
pub fn sess_to_bin(sessid : &str) -> Result<Vec<u8>> {
    use data_encoding::base16;
    if sessid.len() == SESSID_BITS/4 {
        base16::decode(sessid.as_bytes()).chain_err(|| ErrorKind::BadSessId)
    } else {
        Err(ErrorKind::BadSessId.to_err())
    }
}


pub fn check_session(conn : &PgConnection, session_id : &str, ip : IpAddr) -> Result<(User, Session)> {
    use schema::{users, sessions};
    use diesel::ExpressionMethods;
    use diesel::query_builder::AsChangeset;
    use diesel::result::Error::NotFound;

    let new_sessid = fresh_sessid()?;

    let ip_as_bytes = match ip {
        IpAddr::V4(ip) => { ip.octets()[..].to_vec() },
        IpAddr::V6(ip) => { ip.octets()[..].to_vec() },
    };

    let fresh_sess = RefreshSession {
        sess_id: &new_sessid,
        last_ip: ip_as_bytes,
        last_seen: chrono::UTC::now(),
    };

    let session : Session = diesel::update(
            sessions::table
            .filter(sessions::sess_id.eq(sess_to_bin(session_id)?))
        )
        .set(fresh_sess.as_changeset())
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchSess),
                e => e.caused_err(|| "Couldn't update the session."),
        })?;

    let user = users::table
        .filter(users::id.eq(session.user_id))
        .first(conn)
        .chain_err(|| "Couldn't get the user.")?;

    Ok((user, session))
} 

pub fn end_session(conn : &PgConnection, session_id : &str) -> Result<()> {
    use schema::sessions;

    diesel::delete(sessions::table
        .filter(sessions::sess_id.eq(sess_to_bin(session_id).chain_err(|| "Session ID was malformed!")?)))
        .execute(conn)
        .chain_err(|| "Couldn't end the session.")?;
    Ok(())
} 

fn fresh_sessid() -> Result<[u8; SESSID_BITS/8]> {
    use rand::{Rng, OsRng};
    let mut session_id = [0_u8; SESSID_BITS/8];
    OsRng::new().chain_err(|| "Unable to connect to the system random number generator!")?.fill_bytes(&mut session_id);
    Ok(session_id)
}

pub fn start_session(conn : &PgConnection, user : &User, ip : IpAddr) -> Result<Session> {
    use schema::sessions;

    let new_sessid = fresh_sessid()?;

    let ip_as_bytes = match ip {
        IpAddr::V4(ip) => { ip.octets()[..].to_vec() },
        IpAddr::V6(ip) => { ip.octets()[..].to_vec() },
    };

    let new_sess = NewSession {
        sess_id: &new_sessid,
        user_id: user.id,
        last_ip: ip_as_bytes,
    };

    diesel::insert(&new_sess)
        .into(sessions::table)
        .get_result(conn)
        .chain_err(|| "Couldn't start a session!") // TODO if the session id already exists, this is going to fail? (A few-in-a 2^128 change, though...)
}

pub fn add_pending_email_confirm(conn : &PgConnection, email : &str) -> Result<String> {
    use schema::pending_email_confirms;
    let secret = data_encoding::base64url::encode(&fresh_sessid()?[..]);
    {
        let confirm = NewPendingEmailConfirm {
            email: email,
            secret: secret.as_ref(),
        };
        diesel::insert(&confirm)
            .into(pending_email_confirms::table)
            .execute(conn)
            .chain_err(|| "Error :(")?;
    }
    Ok(secret)
}

pub fn check_pending_email_confirm(conn : &PgConnection, secret : &str) -> Result<String> {
    use schema::pending_email_confirms;
    let confirm : PendingEmailConfirm = pending_email_confirms::table
        .filter(pending_email_confirms::secret.eq(secret))
        .first(conn)
        .chain_err(|| "No such secret/email found :(")?;
    Ok(confirm.email)
}

pub fn complete_pending_email_confirm(conn : &PgConnection, password : &str, secret : &str) -> Result<User> {
    use schema::pending_email_confirms;
    let email = check_pending_email_confirm(&*conn, secret)?;
    let user = add_user(&*conn, &email, password)?;

    diesel::delete(pending_email_confirms::table
        .filter(pending_email_confirms::secret.eq(secret)))
        .execute(conn)
        .chain_err(|| "Couldn't delete the pending request.")?;

    Ok(user)
}

#[derive(Debug)]
pub struct Fieldset {
    pub q_variants: Vec<(PathBuf, mime::Mime)>,
    pub answer_audio: Option<(PathBuf, mime::Mime)>,
    pub answer_text: String,
}


pub fn create_quiz(conn : &PgConnection, data: (String, String, String, Vec<Fieldset>)) -> Result<QuizQuestion> {
    use schema::{quiz_questions, question_answers, question_audio, narrators};

    println!("Creating quiz!");

    let new_quiz = NewQuizQuestion { skill_id: None, q_name: &data.0, q_explanation: &data.1 };

    let quiz : QuizQuestion = diesel::insert(&new_quiz)
        .into(quiz_questions::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new question!")?;

    println!("{:?}", &quiz);


    let new_narrator = NewNarrator { name: "anonymous" };
    
    let narrator : Narrator = diesel::insert(&new_narrator)
        .into(narrators::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new narrator!")?;


    for fieldset in &data.3 {
        let a_audio = &fieldset.answer_audio;

        let path;
        if let &Some(ref path_mime) = a_audio {
            path = path_mime.0.to_str().expect("this is an ascii path!");
        } else {
            path = "";
        }

        let new_answer = NewAnswer { question_id: quiz.id, answer_text: &fieldset.answer_text, answer_audio: path };

        let answer : Answer = diesel::insert(&new_answer)
            .into(question_answers::table)
            .get_result(&*conn)
            .chain_err(|| "Couldn't create a new answer!")?;

        println!("{:?}", &answer);

        for q_audio in &fieldset.q_variants {

            let path = q_audio.0.to_str().expect("this is an ascii path");
            let new_q_audio = NewQuestionAudio {answer_id: answer.id, narrator_id: narrator.id, audio_file: path};

            let q_audio : QuestionAudio = diesel::insert(&new_q_audio)
                .into(question_audio::table)
                .get_result(&*conn)
                .chain_err(|| "Couldn't create a new question_audio!")?;

            println!("{:?}", &q_audio);
        }
        
    }
    Ok(quiz)
}

pub fn get_new_quiz(conn : &PgConnection, user : &User) -> Result<String> {
    Ok("juu".into())
}

pub fn get_line_file(conn : &PgConnection, line_id : &str) -> (String, mime::Mime) {
    ("cards/card00001/voice00001/card1-01.mp3".into(), mime!(Audio/Mpeg)) // TODO
}
