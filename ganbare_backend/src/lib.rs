#![recursion_limit = "512"]
#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]
#![feature(proc_macro, custom_derive, custom_attribute, plugin)]
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
pub mod password;
pub mod errors;
pub mod user;
pub mod session;
pub mod audio;
pub mod quiz;
pub mod manage;


pub use models::*;
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
        Some(e) => match e.1.event_finish {
            Some(_) => true,
            None => false,
        },
        None => false,
    })
}


pub fn initiate(conn: &PgConnection, event_name: &str, user: &User) -> Result<Option<(Event, EventExperience)>> {
    use schema::{event_experiences, events};

    if let Some((ev, exp)) = state(conn, event_name, user)? { 
        return Ok(Some((ev, exp)))
    };

    let ev: Event = events::table
        .filter(events::name.eq(event_name))
        .get_result(conn)?;

    let exp: EventExperience = diesel::insert(&NewEventExperience {user_id: user.id, event_id: ev.id })
        .into(event_experiences::table)
        .get_result(conn)?;

    Ok(Some((ev, exp)))
}

pub fn require_done(conn: &PgConnection, event_name: &str, user: &User) -> Result<(Event, EventExperience)> {

    let ev_state = state(conn, event_name, user)?;

    if let  Some(ev_exp@(_, EventExperience { event_finish: Some(_), .. })) = ev_state {
        Ok(ev_exp)
    } else {
        Err(ErrorKind::AccessDenied.to_err())
    }
}

pub fn require_ongoing(conn: &PgConnection, event_name: &str, user: &User) -> Result<(Event, EventExperience)> {

    let ev_state = state(conn, event_name, user)?;

    if let Some(ev_exp@(_, EventExperience { event_finish: None, .. })) = ev_state {
        Ok(ev_exp)
    } else {
        Err(ErrorKind::AccessDenied.to_err())
    }
}


pub fn set_done(conn: &PgConnection, event_name: &str, user: &User) -> Result<Option<(Event, EventExperience)>> {
    use schema::{event_experiences};

    if let Some((ev, mut exp)) = state(conn, event_name, user)? {
        exp.event_finish = Some(chrono::UTC::now());
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

pub fn save_userdata(conn: &PgConnection, event: &Event, user: &User, key: Option<&str>, data: &str) -> Result<EventUserdata> {
    use schema::event_userdata;

    match key {
        None => Ok(diesel::insert(&NewEventUserdata { event_id: event.id, user_id: user.id, key, data })
                    .into(event_userdata::table)
                    .get_result(conn)?),
        Some(k) => {
            let result = diesel::update(
                        event_userdata::table
                            .filter(event_userdata::event_id.eq(event.id))
                            .filter(event_userdata::user_id.eq(user.id))
                            .filter(event_userdata::key.eq(k))
                    )
                    .set(&UpdateEventUserdata { data })
                    .get_result(conn)
                    .optional()?;
            if let Some(userdata) = result {
                Ok(userdata)
            } else {
                Ok(diesel::insert(&NewEventUserdata { event_id: event.id, user_id: user.id, key, data })
                    .into(event_userdata::table)
                    .get_result(conn)?)
            }
        },
    }
}

}
