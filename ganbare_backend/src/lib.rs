#![recursion_limit = "512"]
#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, binary_macros, dotenv_macros)]

#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate mime;
#[macro_use]
extern crate lazy_static;

extern crate try_map;
extern crate tempdir;
extern crate time;
extern crate crypto;
pub extern crate chrono;
extern crate rand;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate unicode_normalization;
extern crate regex;
extern crate hyper;

pub use try_map::{FallibleMapExt, FlipResultExt};

pub use diesel::prelude::*;

pub use diesel::pg::PgConnection;


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}

pub mod schema;
pub mod models;
pub mod event;
pub mod email;
pub mod password;
pub mod errors;
pub mod user;
pub mod session;
pub mod audio;
pub mod quiz;
pub mod manage;
pub mod test;


pub use models::*;
pub use errors::*;



pub mod sql {
    no_arg_sql_function!(random, ::diesel::types::Numeric);
}



pub mod db {
    use super::*;

    pub fn check(conn: &PgConnection) -> Result<bool> {
        run_migrations(conn).chain_err(|| "Couldn't run the migrations.")?;
        let first_user: Option<User> = schema::users::table.first(conn)
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

        let count: i64 = schema::users::table.count()
            .get_result(conn)?;

        Ok(count > 0)
    }

    pub fn connect(database_url: &str) -> Result<PgConnection> {
        PgConnection::establish(database_url).chain_err(|| "Error connecting to database!")
    }

}











pub mod skill {
    use super::*;

    pub fn get_create_by_name(conn: &PgConnection, skill_summary: &str) -> Result<SkillNugget> {
        use schema::skill_nuggets;

        let skill_nugget: Option<SkillNugget> =
            skill_nuggets::table.filter(skill_nuggets::skill_summary.eq(skill_summary))
                .get_result(conn)
                .optional()
                .chain_err(|| "Database error with skill_nuggets!")?;

        Ok(match skill_nugget {
            Some(nugget) => nugget,
            None => {
            diesel::insert(&NewSkillNugget{ skill_summary })
                .into(skill_nuggets::table)
                .get_result(conn)
                .chain_err(|| "Database error!")?
        }
        })
    }

    pub fn get_skill_data(conn: &PgConnection, user_id: i32) -> Result<Vec<(SkillNugget, SkillData)>> {
        use schema::{skill_nuggets, skill_data};

        let data: Vec<(SkillNugget, SkillData)> = skill_nuggets::table
            .inner_join(skill_data::table)
            .filter(skill_data::user_id.eq(user_id))
            .get_results(conn)?;

        Ok(data)
    }

    pub fn get_skill_nuggets(conn: &PgConnection)
                             -> Result<Vec<(SkillNugget,
                                            (Vec<Word>,
                                             Vec<(QuizQuestion, Vec<Answer>)>,
                                             Vec<(Exercise, Vec<ExerciseVariant>)>))>> {
        use schema::{skill_nuggets, quiz_questions, question_answers, words, exercises,
                     exercise_variants};

        let nuggets: Vec<SkillNugget> =
            skill_nuggets::table.order(skill_nuggets::skill_summary.asc()).get_results(conn)?;

        let words = Word::belonging_to(&nuggets)
            .order(words::id.asc())
            .load::<Word>(conn)?
            .grouped_by(&nuggets);

        let questions = QuizQuestion::belonging_to(&nuggets).order(quiz_questions::id.asc())
            .load::<QuizQuestion>(conn)?;

        let q_answers = Answer::belonging_to(&questions)
            .order(question_answers::id.asc())
            .load::<Answer>(conn)?
            .grouped_by(&questions);

        let qs_and_as = questions.into_iter()
            .zip(q_answers.into_iter())
            .collect::<Vec<_>>()
            .grouped_by(&nuggets);

        let exercises = Exercise::belonging_to(&nuggets).order(exercises::id.asc())
            .load::<Exercise>(conn)?;

        let e_answers = ExerciseVariant::belonging_to(&exercises)
            .order(exercise_variants::id.asc())
            .load::<ExerciseVariant>(conn)?
            .grouped_by(&exercises);

        let es_and_as = exercises.into_iter()
            .zip(e_answers.into_iter())
            .collect::<Vec<_>>()
            .grouped_by(&nuggets);

        let cards = words.into_iter()
            .zip(qs_and_as.into_iter().zip(es_and_as.into_iter()))
            .map(|(ws, (qs, es))| (ws, qs, es));
        let all = nuggets.into_iter().zip(cards).collect();
        Ok(all)
    }

    pub fn log_by_id(conn: &PgConnection,
                     user_id: i32,
                     skill_id: i32,
                     level_increment: i32)
                     -> Result<SkillData> {
        use schema::skill_data;

        debug!("Skill bump! Skill: {} Of user: {} Bumped by: {}",
               skill_id,
               user_id,
               level_increment);

        let skill_data: Option<SkillData> =
            skill_data::table.filter(skill_data::user_id.eq(user_id))
                .filter(skill_data::skill_nugget.eq(skill_id))
                .get_result(conn)
                .optional()?;
        Ok(if let Some(skill_data) = skill_data {
            diesel::update(skill_data::table
                            .filter(skill_data::user_id.eq(user_id))
                            .filter(skill_data::skill_nugget.eq(skill_id)))
                .set(skill_data::skill_level.eq(skill_data.skill_level + level_increment))
                .get_result(conn)?
        } else {
            diesel::insert(&SkillData {
                    user_id: user_id,
                    skill_nugget: skill_id,
                    skill_level: level_increment,
                }).into(skill_data::table)
                .get_result(conn)?
        })

    }

    pub fn remove(conn: &PgConnection, id: i32) -> Result<Option<SkillNugget>> {
        use schema::skill_nuggets;

        let skill: Option<SkillNugget> =
            diesel::delete(skill_nuggets::table.filter(skill_nuggets::id.eq(id))).get_result(conn)
                .optional()?;

        Ok(skill)
    }

}
