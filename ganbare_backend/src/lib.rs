#![recursion_limit = "512"]
#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]
#![feature(plugin)]
#![plugin(binary_macros, dotenv_macros)]


#[macro_use]
pub extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;
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
extern crate reqwest;
extern crate dotenv;
extern crate cookie;

pub use try_map::{FallibleMapExt, FlipResultExt};
use std::sync::atomic::{Ordering, AtomicBool};

pub use diesel::prelude::*;
pub use diesel::pg::PgConnection as DieselPgConnection;
pub type ConnManager = r2d2_diesel::ConnectionManager<DieselPgConnection>;
pub type Connection = r2d2::PooledConnection<ConnManager>;
pub use diesel::Connection as ConnectionTrait;


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}


lazy_static! {

    pub static ref DB_INSTALLED: AtomicBool = {
        AtomicBool::new(false)
    };

    pub static ref PERF_TRACE: bool = {
        dotenv::dotenv().ok();
        std::env::var("GANBARE_PERF_TRACE")
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false)
    };
}

#[macro_export]
macro_rules! time_it {
    ($comment:expr , $code:expr) => {
        {
            if *PERF_TRACE {
                use std::time::Instant;
                let start = Instant::now();
    
                let res = $code;
    
                let end = Instant::now();
                let lag = end.duration_since(start);
                debug!("{}:{} time_it {} took {}s {}ms!",
                    file!(),
                    line!(),
                    $comment,
                    lag.as_secs(),
                    lag.subsec_nanos()/1_000_000);
                res
            } else {
                $code
            }
        }
    };
    ($code:expr) => { time_it!("", $code) };
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
    use ::diesel::types;

    no_arg_sql_function!(random, types::Numeric);
    sql_function!(lower, lower_sql_function, (string: types::Nullable<types::Text>) -> types::Nullable<types::Text>);
}



pub mod db {
    use super::*;

    pub fn check(conn: &Connection) -> Result<bool> {
        run_migrations(conn).chain_err(|| "Couldn't run the migrations.")?;
        init_check_is_installed(conn)?;
        Ok(is_installed())
    }

    #[cfg(not(debug_assertions))]
    embed_migrations!();

    #[cfg(not(debug_assertions))]
    fn run_migrations(conn: &Connection) -> Result<()> {
        embedded_migrations::run(&**conn)?;
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn run_migrations(conn: &Connection) -> Result<()> {
        diesel::migrations::run_pending_migrations(&**conn)?;
        info!("Migrations checked.");
        Ok(())
    }

    pub fn is_installed() -> bool {
        DB_INSTALLED.load(Ordering::Acquire)
    }

    pub fn set_installed() {
        DB_INSTALLED.store(true, Ordering::Release);
    }

    fn init_check_is_installed(conn: &Connection) -> Result<()> {

        let count: i64 = schema::users::table.count()
            .get_result(&**conn)?;

        DB_INSTALLED.store(count > 0, Ordering::Release);
        Ok(())
    }

    pub fn connect(database_url: &str) -> Result<DieselPgConnection> {

        DieselPgConnection::establish(database_url).chain_err(|| "Error connecting to database!")
    }

}











pub mod skill {
    use super::*;

    pub fn get_create_by_name(conn: &Connection, skill_summary: &str) -> Result<SkillNugget> {
        use schema::skill_nuggets;

        let skill_nugget: Option<SkillNugget> =
            skill_nuggets::table.filter(skill_nuggets::skill_summary.eq(skill_summary))
                .get_result(&**conn)
                .optional()
                .chain_err(|| "Database error with skill_nuggets!")?;

        Ok(match skill_nugget {
            Some(nugget) => nugget,
            None => {
            diesel::insert(&NewSkillNugget{ skill_summary })
                .into(skill_nuggets::table)
                .get_result(&**conn)
                .chain_err(|| "Database error!")?
        }
        })
    }

    pub fn get_skill_data(conn: &Connection, user_id: i32) -> Result<Vec<(SkillNugget, SkillData)>> {
        use schema::{skill_nuggets, skill_data};

        let data: Vec<(SkillNugget, SkillData)> = skill_nuggets::table
            .inner_join(skill_data::table)
            .filter(skill_data::user_id.eq(user_id))
            .get_results(&**conn)?;

        Ok(data)
    }

    pub fn get_asked_items(conn: &Connection, user_id: i32)
        -> Result<(
            Vec<(DueItem, QuestionData, QuizQuestion)>,
            Vec<(DueItem, ExerciseData, Exercise)>,
            Vec<(PendingItem, WAskedData, Word)>
        )> {
        use schema::{due_items, question_data, exercise_data, quiz_questions,
            exercises, words, pending_items, w_asked_data};

        let data_q: Vec<(DueItem, QuestionData)> = due_items::table
            .inner_join(question_data::table)
            .filter(due_items::user_id.eq(user_id))
            .get_results(&**conn)?;

        let mut questions: Vec<QuizQuestion> = vec![];

        for &(_, ref data) in &data_q {
            let q = quiz_questions::table
                .filter(quiz_questions::id.eq(data.question_id))
                .get_result(&**conn)?;
            questions.push(q);
        }

        let data_e: Vec<(DueItem, ExerciseData)> = due_items::table
            .inner_join(exercise_data::table)
            .filter(due_items::user_id.eq(user_id))
            .get_results(&**conn)?;

        let mut exercises: Vec<Exercise> = vec![];

        for &(_, ref data) in &data_e {
            exercises.push(exercises::table
                .filter(exercises::id.eq(data.exercise_id))
                .get_result(&**conn)?);
        }

        let data_w: Vec<(PendingItem, WAskedData)> = pending_items::table
            .inner_join(w_asked_data::table)
            .filter(pending_items::user_id.eq(user_id).and(pending_items::test_item.eq(false)))
            .get_results(&**conn)?;

        let mut words: Vec<Word> = vec![];

        for &(_, ref data) in &data_w {
            words.push(words::table
                .filter(words::id.eq(data.word_id))
                .get_result(&**conn)?);
        }

        let q = data_q.into_iter().zip(questions.into_iter()).map(|((a, b), c)| (a, b, c)).collect();
        let e = data_e.into_iter().zip(exercises.into_iter()).map(|((a, b), c)| (a, b, c)).collect();
        let w = data_w.into_iter().zip(words.into_iter()).map(|((a, b), c)| (a, b, c)).collect();

        Ok((q, e, w))
    }

    pub fn get_skill_nuggets(conn: &Connection)
                             -> Result<Vec<(SkillNugget,
                                            (Vec<Word>,
                                             Vec<(QuizQuestion, Vec<Answer>)>,
                                             Vec<(Exercise, Vec<ExerciseVariant>)>))>> {
        use schema::{skill_nuggets, quiz_questions, question_answers, words, exercises,
                     exercise_variants};

        let nuggets: Vec<SkillNugget> =
            skill_nuggets::table.order(skill_nuggets::skill_summary.asc()).get_results(&**conn)?;

        let words = Word::belonging_to(&nuggets)
            .order(words::id.asc())
            .load::<Word>(&**conn)?
            .grouped_by(&nuggets);

        let questions = QuizQuestion::belonging_to(&nuggets).order(quiz_questions::id.asc())
            .load::<QuizQuestion>(&**conn)?;

        let q_answers = Answer::belonging_to(&questions)
            .order(question_answers::id.asc())
            .load::<Answer>(&**conn)?
            .grouped_by(&questions);

        let qs_and_as = questions.into_iter()
            .zip(q_answers.into_iter())
            .collect::<Vec<_>>()
            .grouped_by(&nuggets);

        let exercises = Exercise::belonging_to(&nuggets).order(exercises::id.asc())
            .load::<Exercise>(&**conn)?;

        let e_answers = ExerciseVariant::belonging_to(&exercises)
            .order(exercise_variants::id.asc())
            .load::<ExerciseVariant>(&**conn)?
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

    pub fn log_by_id(conn: &Connection,
                     user_id: i32,
                     skill_id: i32,
                     level_increment: i32)
                     -> Result<SkillData> {
        use schema::skill_data;

        let skill_data: Option<SkillData> =
            skill_data::table.filter(skill_data::user_id.eq(user_id))
                .filter(skill_data::skill_nugget.eq(skill_id))
                .get_result(&**conn)
                .optional()?;
        Ok(if let Some(skill_data) = skill_data {
            diesel::update(skill_data::table
                            .filter(skill_data::user_id.eq(user_id))
                            .filter(skill_data::skill_nugget.eq(skill_id)))
                .set(skill_data::skill_level.eq(skill_data.skill_level + level_increment))
                .get_result(&**conn)?
        } else {
            diesel::insert(&SkillData {
                    user_id: user_id,
                    skill_nugget: skill_id,
                    skill_level: level_increment,
                }).into(skill_data::table)
                .get_result(&**conn)?
        })

    }

    pub fn remove(conn: &Connection, id: i32) -> Result<Option<SkillNugget>> {
        use schema::skill_nuggets;

        let skill: Option<SkillNugget> =
            diesel::delete(skill_nuggets::table.filter(skill_nuggets::id.eq(id))).get_result(&**conn)
                .optional()?;

        Ok(skill)
    }

}
