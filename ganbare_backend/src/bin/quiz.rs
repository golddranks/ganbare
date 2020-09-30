extern crate ganbare_backend;
extern crate diesel;

#[macro_use]
extern crate clap;
extern crate rpassword;
extern crate lettre;
extern crate dotenv;
extern crate handlebars;
extern crate r2d2;

use ganbare_backend::{Connection, ConnManager};
use ganbare_backend::errors::*;
use ganbare_backend::models::*;
use diesel::LoadDsl;
#[macro_use]
extern crate lazy_static;


lazy_static! {

    static ref DATABASE_URL : String = {
        dotenv::dotenv().ok();
        std::env::var("GANBARE_DATABASE_URL")
            .expect(
            "GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)"
            )
    };

}


pub fn list_skillnuggets(conn: &Connection) -> Result<Vec<SkillNugget>> {
    use ganbare_backend::schema::skill_nuggets::dsl::*;

    skill_nuggets.load::<SkillNugget>(&**conn).chain_err(|| "Can't load")

}

pub fn list_questions(conn: &Connection) -> Result<Vec<QuizQuestion>> {
    use ganbare_backend::schema::quiz_questions::dsl::*;

    quiz_questions.load::<QuizQuestion>(&**conn).chain_err(|| "Can't load")
}


fn main() {
    use clap::*;

    let matches = App::new("ganba.re quiz interface")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .subcommand(SubCommand::with_name("lss").about("List all skill nuggets"))
        .subcommand(SubCommand::with_name("lsq").about("List all questions"))
        .subcommand(SubCommand::with_name("addq").about("Add a question"))
        .get_matches();

    let manager = ConnManager::new(DATABASE_URL.as_str());
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool.");
    let conn = pool.get().unwrap();

    match matches.subcommand() {
        ("lss", Some(_)) => {
            let items = list_skillnuggets(&conn).unwrap();
            println!("{} skill nuggets found:", items.len());
            for i in items {
                println!("{:?}", i);
            }
        }
        ("lsq", Some(_)) => {
            let items = list_questions(&conn).unwrap();
            println!("{} questions found:", items.len());
            for i in items {
                println!("{:?}", i);
            }
        }
        _ => {
            // clap should exit before reaching here if none of the subcommands are entered.
            unreachable!();
        }
    }
}
