#![recursion_limit = "512"]
#![feature(inclusive_range_syntax)]
#![feature(proc_macro)]
#![feature(field_init_shorthand)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, binary_macros, dotenv_macros)]


#[macro_use] pub extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
#[macro_use] extern crate mime;

extern crate time;
extern crate crypto;
pub extern crate chrono;
extern crate rand;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate unicode_normalization;

type DateTimeUTC = chrono::DateTime<chrono::UTC>;

pub use diesel::prelude::*;

pub use diesel::pg::PgConnection;


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}


pub mod schema;
pub mod models;
pub mod email;
pub use models::*;
pub mod password;
pub mod errors {

    error_chain! {
        foreign_links {
            ::std::str::ParseBoolError, ParseBoolError;
            ::std::env::VarError, VarError;
            ::std::num::ParseIntError, ParseIntError;
            ::std::num::ParseFloatError, ParseFloatError;
            ::std::io::Error, StdIoError;
            ::diesel::result::Error, DieselError;
            ::diesel::migrations::RunMigrationsError, DieselMigrationError;
        }
        errors {
            InvalidInput {
                description("Provided input is invalid.")
                display("Provided input is invalid.")
            }
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

pub use errors::*;












pub mod db {
    use super::*;

pub fn check(conn: &PgConnection) -> Result<bool> {
    run_migrations(conn).chain_err(|| "Couldn't run the migrations.")?;
    let first_user: Option<User> = schema::users::table
        .first(conn)
        .optional()
        .chain_err(|| "Couldn't query for the admin user.")?;

    Ok(first_user.is_some())
}

#[cfg(not(debug_assertions))]
embed_migrations!();

#[cfg(not(debug_assertions))]
fn run_migrations(conn: &PgConnection) -> Result<()> {
    embedded_migrations::run(conn)?;
    Ok(())
}

#[cfg(debug_assertions)]
fn run_migrations(conn: &PgConnection) -> Result<()> {
    diesel::migrations::run_pending_migrations(conn)?;
    info!("Migrations checked.");
    Ok(())
}

pub fn is_installed(conn: &PgConnection) -> Result<bool> {

    let count: i64 = schema::users::table
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}

pub fn connect(database_url: &str) -> Result<PgConnection> {
    PgConnection::establish(database_url)
        .chain_err(|| "Error connecting to database!")
}

}












pub mod user;




pub mod session;







pub mod audio;




pub mod skill {
    use super::*;

pub fn get_create_by_name(conn : &PgConnection, skill_summary: &str) -> Result<SkillNugget> {
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

pub fn get_skill_nuggets(conn : &PgConnection) -> Result<Vec<(SkillNugget, (Vec<Word>, Vec<(QuizQuestion, Vec<Answer>)>))>> {
    use schema::{skill_nuggets, quiz_questions, question_answers, words};
    let nuggets: Vec<SkillNugget> = skill_nuggets::table.get_results(conn)?;
    let qs = if nuggets.len() > 0 {
        QuizQuestion::belonging_to(&nuggets).order(quiz_questions::id.asc()).load::<QuizQuestion>(conn)?
    } else { vec![] };

    // FIXME checking this special case until the panicking bug in Diesel is fixed
    let aas = if qs.len() > 0 {
        Answer::belonging_to(&qs).order(question_answers::id.asc()).load::<Answer>(conn)?.grouped_by(&qs)
    } else { vec![] };

    let qs_and_as = qs.into_iter().zip(aas.into_iter()).collect::<Vec<_>>().grouped_by(&nuggets);

    // FIXME checking this special case until the panicking bug in Diesel is fixed
    let ws = if nuggets.len() > 0 {
        Word::belonging_to(&nuggets).order(words::id.asc()).load::<Word>(conn)?.grouped_by(&nuggets)
    } else { vec![] };

    let cards = ws.into_iter().zip(qs_and_as.into_iter());
    let all = nuggets.into_iter().zip(cards).collect();
    Ok(all)
}

pub fn log_by_id(conn : &PgConnection, user : &User, skill_id: i32, level_increment: i32) -> Result<SkillData> {
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

}






pub mod quiz;




pub mod manage;







pub mod event {

    use super::*;

pub fn state(conn: &PgConnection, event_name: &str, user: &User) -> Result<Option<(Event, EventExperience)>> {
    use schema::{event_experiences, events};

    let ok = events::table
        .inner_join(event_experiences::table)
        .filter(event_experiences::user_id.eq(user.id))
        .filter(events::name.eq(event_name))
        .get_result(conn)
        .optional()?;
    Ok(ok)
}


pub fn is_done(conn: &PgConnection, event_name: &str, user: &User) -> Result<bool> {
    let state = state(conn, event_name, user)?;
    Ok(match state {
        Some(e) => match e.1.event_time {
            Some(_) => true,
            None => false,
        },
        None => false,
    })
}


pub fn initiate(conn: &PgConnection, event_name: &str, user: &User) -> Result<Option<(Event, EventExperience)>> {
    use schema::{event_experiences, events};

    if let Some((ev, exp)) = state(conn, event_name, user)? { return Ok(Some((ev, exp))) };

    let ev: Event = events::table
        .filter(events::name.eq(event_name))
        .get_result(conn)?;

    let exp: EventExperience = diesel::insert(&EventExperience {user_id: user.id, event_id: ev.id, event_time: None})
        .into(event_experiences::table)
        .get_result(conn)?;

    Ok(Some((ev, exp)))
}


pub fn set_done(conn: &PgConnection, event_name: &str, user: &User) -> Result<Option<(Event, EventExperience)>> {
    use schema::{event_experiences};

    if let Some((ev, mut exp)) = state(conn, event_name, user)? {
        exp.event_time = Some(chrono::UTC::now());
        diesel::update(
                event_experiences::table
                    .filter(event_experiences::event_id.eq(ev.id))
                    .filter(event_experiences::user_id.eq(user.id))
                )
            .set(&exp)
            .execute(conn)?;
        Ok(Some((ev, exp)))
    } else {
        Ok(None)
    }
}

}
