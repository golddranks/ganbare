#![recursion_limit = "1024"]
#![feature(proc_macro)]
#![feature(field_init_shorthand)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros, binary_macros)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate mime;

extern crate time;
extern crate dotenv;
extern crate crypto;
extern crate chrono;
extern crate rand;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate pencil;



use rand::thread_rng;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::net::IpAddr;
use std::path::PathBuf;

pub use diesel::pg::PgConnection;


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}


pub mod schema;
pub mod models;
use models::*;
pub mod password;
pub mod errors {

    error_chain! {
        foreign_links {
            ::std::num::ParseIntError, ParseIntError;
            ::std::num::ParseFloatError, ParseFloatError;
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
            FileNotFound {
                description("Can't find that file!")
                display("Can't find that file!")
            }
            DatabaseOdd {
                description("There's something wrong with the contents of the DB vs. how it should be!")
                display("There's something wrong with the contents of the DB vs. how it should be!")
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
    use schema::{users, passwords, user_metrics};

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

    diesel::insert(&NewUserMetrics{ id: user.id })
        .into(user_metrics::table)
        .execute(conn)
        .chain_err(|| "Couldn't insert the new password into user metrics!")?;

    Ok(user)
}

pub fn set_password(conn : &PgConnection, user_email : &str, password: &str) -> Result<User> {
    use schema::{users, passwords};

    let (u, p) : (User, Option<Password>) = users::table
        .left_outer_join(passwords::table)
        .filter(users::email.eq(user_email))
        .first(&*conn)
        .map_err(|e| e.caused_err(|| "Error when trying to retrieve user!"))?;
    if p.is_none() {

        let pw = password::set_password(password).chain_err(|| "Setting password didn't succeed!")?;

        diesel::insert(&pw.into_db(u.id))
            .into(passwords::table)
            .execute(conn)
            .chain_err(|| "Couldn't insert the new password into database!")?;

        Ok(u)
    } else {
        Err("Password already set!".into())
    }
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

pub fn change_password(conn : &PgConnection, user_id : i32, new_password : &str) -> Result<()> {

    let pw = password::set_password(new_password).chain_err(|| "Setting password didn't succeed!")?;

    let _ : models::Password = pw.into_db(user_id).save_changes(conn)?;

    Ok(())
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


pub fn check_session(conn : &PgConnection, session_id : &str) -> Result<(User, Session)> {
    use schema::{users, sessions};
    use diesel::ExpressionMethods;
    use diesel::result::Error::NotFound;

    let (session, user) : (Session, User) = sessions::table
        .inner_join(users::table)
        .filter(sessions::sess_id.eq(sess_to_bin(session_id)?))
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchSess),
                e => e.caused_err(|| "Database error?!"),
        })?;

    Ok((user, session))
} 

pub fn refresh_session(conn : &PgConnection, old_session : &Session, ip : IpAddr) -> Result<Session> {
    use schema::{sessions};
    use diesel::ExpressionMethods;
    use diesel::result::Error::NotFound;

    let new_sessid = fresh_sessid()?;

    let ip_as_bytes = match ip {
        IpAddr::V4(ip) => { ip.octets()[..].to_vec() },
        IpAddr::V6(ip) => { ip.octets()[..].to_vec() },
    };

    let fresh_sess = NewSession {
        sess_id: &new_sessid,
        user_id: old_session.user_id,
        started: old_session.started,
        last_seen: chrono::UTC::now(),
        last_ip: ip_as_bytes,
    };

    let session : Session = diesel::insert(&fresh_sess)
        .into(sessions::table)
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchSess),
                e => e.caused_err(|| "Couldn't update the session."),
        })?;

    // This will delete the user's old sessions IDs, but only after the creation of a new one is underway.
    // In continuous usage, the IDs older than 1 minute will be deleted. But if the user doesn't authenticate,
    // for a while, the newest session IDs will remain until the user returns.
    diesel::delete(
            sessions::table
                .filter(sessions::user_id.eq(old_session.user_id))
                .filter(sessions::last_seen.lt(chrono::UTC::now()-chrono::Duration::minutes(1)))
        ).execute(&*conn)
        .map_err(|_| "Can't delete old sessions!")?;

    Ok(session)
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
        started: chrono::UTC::now(),
        last_seen: chrono::UTC::now(),
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

pub fn check_user_group(conn : &PgConnection, user: &User, group_name: &str )  -> Result<bool> {
    use schema::{user_groups, group_memberships};

    let exists : Option<(UserGroup, GroupMembership)> = user_groups::table
        .inner_join(group_memberships::table)
        .filter(group_memberships::user_id.eq(user.id))
        .filter(user_groups::group_name.eq(group_name))
        .get_result(&*conn)
        .optional()
        .chain_err(|| "DB error")?;

    Ok(exists.is_some())
}

#[derive(Debug)]
pub struct Fieldset {
    pub q_variants: Vec<(PathBuf, Option<String>, mime::Mime)>,
    pub answer_audio: Option<(PathBuf, Option<String>, mime::Mime)>,
    pub answer_text: String,
}


fn save_audio_file(path: &mut std::path::PathBuf, orig_filename: &str) -> Result<()> {
    use rand::Rng;
    let mut new_path = std::path::PathBuf::from("audio/");
    let mut filename = "%FT%H-%M-%SZ".to_string();
    filename.extend(thread_rng().gen_ascii_chars().take(10));
    filename.push_str(".");
    filename.push_str(std::path::Path::new(orig_filename).extension().and_then(|s| s.to_str()).unwrap_or("noextension"));
    new_path.push(time::strftime(&filename, &time::now()).unwrap());
    std::fs::rename(&*path, &new_path)?;
    std::mem::swap(path, &mut new_path);
    Ok(())
}

fn get_create_narrator(conn : &PgConnection, mut name: &str) -> Result<Narrator> {
    use schema::narrators;

    let narrator : Option<Narrator> = if name == "" {
        name = "anonymous";
        None
    } else {
         narrators::table
            .filter(narrators::name.eq(name))
            .get_result(&*conn)
            .optional()
            .chain_err(|| "Database error with narrators!")?
    };


    Ok(match narrator {
        Some(narrator) => narrator,
        None => {
            diesel::insert(&NewNarrator{ name })
                .into(narrators::table)
                .get_result(&*conn)
                .chain_err(|| "Database error!")?
        }
    })
}

fn default_narrator_id(conn: &PgConnection, opt_narrator: &mut Option<Narrator>) -> Result<i32> {
    use schema::narrators;

    if let Some(ref narrator) = *opt_narrator {
        Ok(narrator.id)
    } else {

        let new_narrator : Narrator = diesel::insert(&NewNarrator { name: "anonymous" })
            .into(narrators::table)
            .get_result(conn)
            .chain_err(|| "Couldn't create a new narrator!")?;

        println!("{:?}", &new_narrator);
        let narr_id = new_narrator.id;
        *opt_narrator = Some(new_narrator);
        Ok(narr_id)
    }
}

fn new_audio_bundle(conn : &PgConnection, name: &str) -> Result<AudioBundle> {
    use schema::{audio_bundles};
        let bundle: AudioBundle = diesel::insert(&NewAudioBundle { listname: name })
            .into(audio_bundles::table)
            .get_result(&*conn)
            .chain_err(|| "Can't insert a new audio bundle!")?;
        
        println!("{:?}", bundle);

        Ok(bundle)
}

fn get_create_audio_bundle(conn : &PgConnection, listname: &str) -> Result<AudioBundle> {
    use schema::audio_bundles;

    let audio_bundle : Option<AudioBundle> = audio_bundles::table
        .filter(audio_bundles::listname.eq(listname))
        .get_result(&*conn)
        .optional()
        .chain_err(|| "Database error with audio bundles!")?;

    Ok(match audio_bundle {
        Some(audio_bundle) => audio_bundle,
        None => {
            diesel::insert(&NewAudioBundle{ listname })
                .into(audio_bundles::table)
                .get_result(&*conn)
                .chain_err(|| "Database error!")?
        }
    })
}


fn save_audio(conn : &PgConnection, mut narrator: &mut Option<Narrator>, file: &mut (PathBuf, Option<String>, mime::Mime), bundle: &mut Option<AudioBundle>) -> Result<AudioFile> {
    use schema::{audio_files};

    save_audio_file(&mut file.0, file.1.as_ref().map(|s| s.as_str()).unwrap_or(""))?;

    let bundle_id = if let &mut Some(ref bundle) = bundle {
            bundle.id
        } else {
            let new_bundle = new_audio_bundle(&*conn, "")?;
            let bundle_id = new_bundle.id;
            *bundle = Some(new_bundle);
            bundle_id
        };

    let file_path = file.0.to_str().expect("this is an ascii path");
    let mime = &format!("{}", file.2);
    let narrators_id = default_narrator_id(&*conn, &mut narrator)?;
    let new_q_audio = NewAudioFile {narrators_id, bundle_id, file_path, mime};

    let audio_file : AudioFile = diesel::insert(&new_q_audio)
        .into(audio_files::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new audio file!")?;

    println!("{:?}", &audio_file);

    

    Ok(audio_file)
}

fn load_audio_from_bundles(conn : &PgConnection, bundles: &[AudioBundle]) -> Result<Vec<Vec<AudioFile>>> {
    let q_audio_files : Vec<Vec<AudioFile>> = AudioFile::belonging_to(&*bundles)
        .load(&*conn)
        .chain_err(|| "Can't load quiz!")?
        .grouped_by(&*bundles);

    for q in &q_audio_files { // Sanity check
        if q.len() == 0 {
            return Err(ErrorKind::DatabaseOdd.into());
        }
    };
    Ok(q_audio_files)
}

fn load_audio_from_bundle(conn : &PgConnection, bundle_id: i32) -> Result<Vec<AudioFile>> {
    use schema::audio_files;

    let q_audio_files : Vec<AudioFile> = audio_files::table
        .filter(audio_files::bundle_id.eq(bundle_id))
        .get_results(&*conn)
        .chain_err(|| "Can't load quiz!")?;
    Ok(q_audio_files)
}

fn get_create_skill_nugget_by_name(conn : &PgConnection, skill_summary: &str) -> Result<SkillNugget> {
    use schema::skill_nuggets;

    let skill_nugget : Option<SkillNugget> = skill_nuggets::table
        .filter(skill_nuggets::skill_summary.eq(skill_summary))
        .get_result(&*conn)
        .optional()
        .chain_err(|| "Database error with skill_nuggets!")?;

    Ok(match skill_nugget {
        Some(nugget) => nugget,
        None => {
            diesel::insert(&NewSkillNugget{ skill_summary })
                .into(skill_nuggets::table)
                .get_result(&*conn)
                .chain_err(|| "Database error!")?
        }
    })
}

pub struct NewQuestion {
    pub q_name: String,
    pub q_explanation: String,
    pub question_text: String,
    pub skill_nugget: String,
}

pub fn create_quiz(conn : &PgConnection, new_q: NewQuestion, mut answers: Vec<Fieldset>) -> Result<QuizQuestion> {
    use schema::{quiz_questions, question_answers};

    println!("Creating quiz!");

    // Sanity check
    if answers.len() == 0 {
        return Err(ErrorKind::FormParseError.into());
    }
    for a in &answers {
        if a.q_variants.len() == 0 {
            return Err(ErrorKind::FormParseError.into());
        }
    }

    let nugget = get_create_skill_nugget_by_name(&*conn, &new_q.skill_nugget)?;

    let new_quiz = NewQuizQuestion {
        q_name: &new_q.q_name,
        q_explanation: &new_q.q_explanation,
        question_text: &new_q.question_text,
        skill_id: nugget.id,
        skill_level: 1, // FIXME
    };

    let quiz : QuizQuestion = diesel::insert(&new_quiz)
        .into(quiz_questions::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new question!")?;

    println!("{:?}", &quiz);

    let mut narrator = None;

    for fieldset in &mut answers {
        let mut a_bundle = None;
        let a_audio_id = match fieldset.answer_audio {
            Some(ref mut a) => { Some(save_audio(&*conn, &mut narrator, a, &mut a_bundle)?.id) },
            None => { None },
        };
        
        let mut q_bundle = None;
        for mut q_audio in &mut fieldset.q_variants {
            save_audio(&*conn, &mut narrator, &mut q_audio, &mut q_bundle)?;
        }
        let q_bundle = q_bundle.expect("The audio bundle is initialized now.");

        let new_answer = NewAnswer { question_id: quiz.id, answer_text: &fieldset.answer_text, a_audio_bundle: a_audio_id, q_audio_bundle: q_bundle.id };

        let answer : Answer = diesel::insert(&new_answer)
            .into(question_answers::table)
            .get_result(&*conn)
            .chain_err(|| "Couldn't create a new answer!")?;

        println!("{:?}", &answer);

        
    }
    Ok(quiz)
}

fn load_quiz(conn : &PgConnection, id: i32 ) -> Result<Option<(QuizQuestion, Vec<Answer>, Vec<Vec<AudioFile>>)>> {
    use schema::{quiz_questions, question_answers, audio_bundles};
    use diesel::result::Error::NotFound;

    let qq : Option<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.eq(id))
        .get_result(&*conn)
        .map(|r| Some(r))
        .or_else(|e| match e {
            NotFound => Ok(None),
            e => Err(e.caused_err(|| "Can't load quiz!")),
        })?;

    let qq = try_or!{ qq, else return Ok(None) };

    let (aas, q_bundles) : (Vec<Answer>, Vec<AudioBundle>) = question_answers::table
        .inner_join(audio_bundles::table)
        .filter(question_answers::question_id.eq(qq.id))
        .load(&*conn)
        .chain_err(|| "Can't load quiz!")?
        .into_iter().unzip();

    let q_audio_files = load_audio_from_bundles(&*conn, &q_bundles)?;
    
    Ok(Some((qq, aas, q_audio_files)))
}

#[derive(Debug)]
pub enum Answered {
    Word(AnsweredWord),
    Question(AnsweredQuestion),
}

#[derive(Debug)]
pub struct AnsweredWord {
    pub word_id: i32,
    pub time: i32,
    pub times_audio_played: i32,
}

#[derive(Debug)]
pub struct AnsweredQuestion {
    pub question_id: i32,
    pub right_answer_id: i32,
    pub answered_id: Option<i32>,
    pub q_audio_id: i32,
    pub active_answer_time: i32,
    pub full_answer_time: i32,
}

fn log_skill_by_id(conn : &PgConnection, user : &User, skill_id: i32, level_increment: i32) -> Result<SkillData> {
    use schema::{skill_data};

    let skill_data : Option<SkillData> = skill_data::table
                                        .filter(skill_data::user_id.eq(user.id))
                                        .filter(skill_data::skill_nugget.eq(skill_id))
                                        .get_result(conn)
                                        .optional()?;
    Ok(if let Some(skill_data) = skill_data {
        diesel::update(skill_data::table
                            .filter(skill_data::user_id.eq(user.id))
                            .filter(skill_data::skill_nugget.eq(skill_id)))
                .set(skill_data::skill_level.eq(skill_data.skill_level + level_increment))
                .get_result(conn)?
    } else {
        diesel::insert(&SkillData {
            user_id: user.id,
            skill_nugget: skill_id,
            skill_level: level_increment,
        }).into(skill_data::table)
        .get_result(conn)?
    })

}

fn log_answer_question(conn : &PgConnection, user : &User, answer: &AnsweredQuestion) -> Result<QuestionData> {
    use schema::{answer_data, question_data};

    let correct = answer.right_answer_id == answer.answered_id.unwrap_or(-1);

    // Insert the specifics of this answer event
    let answerdata = NewAnswerData {
        user_id: user.id,
        q_audio_id: answer.q_audio_id,
        correct_qa_id: answer.right_answer_id,
        answered_qa_id: answer.answered_id,
        active_answer_time_ms: answer.active_answer_time,
        full_answer_time_ms: answer.full_answer_time,
        correct: correct,
    };

    diesel::insert(&answerdata)
        .into(answer_data::table)
        .execute(conn)
        .chain_err(|| "Couldn't save the answer data to database!")?;

    let questiondata : Option<QuestionData> = question_data::table
                                        .filter(question_data::user_id.eq(user.id))
                                        .filter(question_data::question_id.eq(answer.question_id))
                                        .get_result(&*conn)
                                        .optional()?;

    // Update the data for this question (due date, statistics etc.)
    Ok(if let Some(questiondata) = questiondata {

        let due_delay = if correct { questiondata.due_delay * 2 } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);

        diesel::update(
                question_data::table
                    .filter(question_data::user_id.eq(user.id))
                    .filter(question_data::question_id.eq(answer.question_id))
            )
            .set((
                question_data::due_date.eq(next_due_date),
                question_data::due_delay.eq(due_delay),
                question_data::correct_streak.eq(questiondata.correct_streak + 1),
            ))
            .get_result(conn)
            .chain_err(|| "Couldn't save the question tally data to database!")?

    } else { // New!

        let due_delay = if correct { 30 } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);

        let questiondata = QuestionData {
            user_id: user.id,
            question_id: answer.question_id,
            correct_streak: if correct { 1 } else { 0 },
            due_date: next_due_date,
            due_delay: due_delay,
        };
        diesel::insert(&questiondata)
            .into(question_data::table)
            .get_result(conn)
            .chain_err(|| "Couldn't save the question tally data to database!")?
    })
}

fn log_answer_word(conn : &PgConnection, user : &User, answer: &AnsweredWord) -> Result<()> {
    use schema::{word_data, user_metrics, words};


    let word: Word = words::table.filter(words::id.eq(answer.word_id)).get_result(conn)?;

    // Insert the specifics of this answer event
    let answerdata = NewWordData {
        user_id: user.id,
        word_id: answer.word_id,
        audio_times: answer.times_audio_played,
        answer_time_ms: answer.time,
    };
    diesel::insert(&answerdata)
        .into(word_data::table)
        .execute(conn)
        .chain_err(|| "Couldn't save the answer data to database!")?;

    let mut metrics : UserMetrics = user_metrics::table
        .filter(user_metrics::id.eq(user.id))
        .get_result(&*conn)?;

    metrics.new_words_today += 1;
    metrics.new_words_since_break += 1;
    let _ : UserMetrics = metrics.save_changes(&*conn)?;

    log_skill_by_id(conn, user, word.skill_nugget, 1)?;

    Ok(())
}

fn get_due_questions(conn : &PgConnection, user_id : i32, allow_peeking: bool) -> Result<Vec<(QuizQuestion, QuestionData)>> {
    use diesel::expression::dsl::*;
    use schema::{quiz_questions, question_data};

    let dues = question_data::table
        .select(question_data::question_id)
        .filter(question_data::user_id.eq(user_id))
        .order(question_data::due_date.asc())
        .limit(5);

    let due_questions : Vec<(QuizQuestion, QuestionData)>;
    if allow_peeking { 

        due_questions = quiz_questions::table
            .inner_join(question_data::table)
            .filter(quiz_questions::id.eq(any(dues)))
            .filter(question_data::user_id.eq(user_id))
            .order(question_data::due_date.asc())
            .get_results(&*conn)
            .chain_err(|| "Can't get due question!")?;

    } else {

        due_questions = quiz_questions::table
            .inner_join(question_data::table)
            .filter(quiz_questions::id.eq(any(
                dues.filter(question_data::due_date.lt(chrono::UTC::now()))
            )))
            .filter(question_data::user_id.eq(user_id))
            .order(question_data::due_date.asc())
            .get_results(&*conn)
            .chain_err(|| "Can't get due question!")?;
    };

    Ok(due_questions)
}

fn get_new_questions(conn : &PgConnection, user_id : i32) -> Result<Vec<QuizQuestion>> {
    use diesel::expression::dsl::*;
    use schema::{quiz_questions, question_data, skill_data};
    let dues = question_data::table
        .select(question_data::question_id)
        .filter(question_data::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_questions : Vec<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.ne(all(dues)))
        .filter(quiz_questions::skill_id.eq(any(skills)))
        .limit(5)
        .order(quiz_questions::id.asc())
        .get_results(conn)
        .chain_err(|| "Can't get new question!")?;

    Ok(new_questions)
}

fn get_new_words(conn : &PgConnection, user_id : i32) -> Result<Vec<Word>> {
    use diesel::expression::dsl::*;
    use schema::{words, word_data};

    let seen = word_data::table
        .select(word_data::word_id)
        .filter(word_data::user_id.eq(user_id));

    let new_words : Vec<Word> = words::table
        .filter(words::id.ne(all(seen)))
        .limit(5)
        .order(words::id.asc())
        .get_results(conn)
        .chain_err(|| "Can't get new words!")?;

    Ok(new_words)
}

#[derive(Debug)]
pub enum Card {
    Word((Word, Vec<AudioFile>)),
    Quiz(Quiz),
}

#[derive(Debug)]
pub struct Quiz {
    pub question: QuizQuestion,
    pub question_audio: Vec<AudioFile>,
    pub right_answer_id: i32,
    pub answers: Vec<Answer>,
    pub due_delay: i32,
    pub due_date: Option<chrono::DateTime<chrono::UTC>>,
}

pub fn get_new_quiz(conn : &PgConnection, user : &User) -> Result<Option<Card>> {
    use rand::Rng;
    use schema::user_metrics;

    // Checking due questions first

    let question_data =
    if let Some(&(ref q, ref qdata)) = get_due_questions(&*conn, user.id, false)?.get(0) {
        Some((q.id, qdata.due_delay, Some(qdata.due_date)))

    } else if let Some(q) = get_new_questions(&*conn, user.id)?.get(0) {
        Some((q.id, 0, None))

    } else { None };

    if let Some((question_id, due_delay, due_date)) = question_data {
        let (question, answers, mut qqs) = try_or!{ load_quiz(conn, question_id)?, else return Ok(None) };
        
        let mut rng = rand::thread_rng();
        let random_answer_index = rng.gen_range(0, answers.len());
        let right_answer_id = answers[random_answer_index].id;
        let question_audio = qqs.remove(random_answer_index);
        
        return Ok(Some(Card::Quiz(Quiz{question, question_audio, right_answer_id, answers, due_delay, due_date})))

    }

    // No questions available ATM, checking words

    let metrics : UserMetrics = user_metrics::table.filter(user_metrics::id.eq(user.id)).get_result(&*conn)?;
    
    if metrics.new_words_today <= 18 || metrics.new_words_since_break <= 6 {
        let mut words = get_new_words(&*conn, user.id)?;
        if words.len() > 0 {
            let the_word = words.swap_remove(0);
            let audio_files = load_audio_from_bundle(&*conn, the_word.audio_bundle)?;

            return Ok(Some(Card::Word((the_word, audio_files))));
        }
    }

    // Peeking for the future

    if let Some(&(ref q, ref qdata)) = get_due_questions(&*conn, user.id, true)?.get(0) {
        let (question, answers, mut qqs) = try_or!{ load_quiz(conn, q.id)?, else return Ok(None) };
        
        let mut rng = rand::thread_rng();
        let random_answer_index = rng.gen_range(0, answers.len());
        let right_answer_id = answers[random_answer_index].id;
        let question_audio = qqs.remove(random_answer_index);
        
        return Ok(Some(Card::Quiz(Quiz{question, question_audio, right_answer_id, answers, due_delay: qdata.due_delay, due_date: Some(qdata.due_date)})));
    }
    Ok(None)
}


pub fn get_next_quiz(conn : &PgConnection, user : &User, answer_enum: Answered)
-> Result<Option<Card>> {

    match answer_enum {
        Answered::Word(answer_word) => {
            log_answer_word(&*conn, user, &answer_word)?;
            return get_new_quiz(conn, user);
        },
        Answered::Question(answer) => {
            let q_data = log_answer_question(&*conn, user, &answer)?;
        
            if q_data.correct_streak > 0 { // RIGHT. Get a new question/word.
                return get_new_quiz(conn, user);
        
            } else {            // WROOONG. Ask the same question again.
        
                let (question, answers, mut q_audio_files ) = try_or!{ load_quiz(conn, answer.question_id)?, else return Ok(None) };

                let right_answer_id = answer.right_answer_id;
                
                let (i, _) = answers.iter().enumerate()
                    .find(|&(_, ref qa)| qa.id == right_answer_id )
                    .ok_or_else(|| ErrorKind::DatabaseOdd.to_err())?;

                let question_audio : Vec<AudioFile> = q_audio_files.remove(i);
        
                return Ok(Some(Card::Quiz(
                    Quiz{question, question_audio, right_answer_id, answers, due_delay: q_data.due_delay, due_date: Some(q_data.due_date)}
                    )))
            }
        },
    }

}

pub fn get_line_file(conn : &PgConnection, line_id : i32) -> Result<(String, mime::Mime)> {
    use schema::audio_files::dsl::*;
    use diesel::result::Error::NotFound;

    let file : AudioFile = audio_files
        .filter(id.eq(line_id))
        .get_result(&*conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::FileNotFound),
                e => e.caused_err(|| "Couldn't get the file!"),
        })?;

    Ok((file.file_path, file.mime.parse().expect("The mimetype from the database should be always valid.")))
}

#[derive(Debug)]
pub struct NewWordFromStrings {
    pub word: String,
    pub explanation: String,
    pub nugget: String,
    pub narrator: String,
    pub files: Vec<(PathBuf, Option<String>, mime::Mime)>,
}

pub fn create_word(conn : &PgConnection, w: NewWordFromStrings) -> Result<Word> {
    use schema::{words};

    let nugget = get_create_skill_nugget_by_name(&*conn, &w.nugget)?;

    let mut narrator = Some(get_create_narrator(&*conn, &w.narrator)?);
    let mut bundle = Some(new_audio_bundle(&*conn, &w.word)?);
    for mut file in w.files {
        save_audio(&*conn, &mut narrator, &mut file, &mut bundle)?;
    } 
    let bundle = bundle.expect("The audio bundle is initialized now.");

    let new_word = NewWord {
        word: &w.word,
        explanation: &w.explanation,
        audio_bundle: bundle.id,
        skill_nugget: nugget.id,
    };

    let word = diesel::insert(&new_word)
        .into(words::table)
        .get_result(conn)
        .chain_err(|| "Can't insert a new word!")?;

    Ok(word)
}
