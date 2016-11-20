extern crate ganbare;
extern crate diesel;

#[macro_use]
extern crate clap;
extern crate rpassword;
extern crate lettre;
extern crate dotenv;
extern crate handlebars;
extern crate rustc_serialize;


use ganbare::*;
use diesel::prelude::*;
use ganbare::errors::*;
use ganbare::models::*;
#[macro_use]  extern crate lazy_static;


lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

}


pub fn list_skillnuggets(conn : &PgConnection) -> Result<Vec<SkillNugget>> {
    use ganbare::schema::skill_nuggets::dsl::*;
 
    skill_nuggets.load::<SkillNugget>(conn).chain_err(|| "Can't load")

}

pub fn list_questions(conn : &PgConnection) -> Result<Vec<QuizQuestion>> {
    use ganbare::schema::quiz_questions::dsl::*;

    quiz_questions.load::<QuizQuestion>(conn).chain_err(|| "Can't load")
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
    let conn = db_connect(&*DATABASE_URL).unwrap();
    match matches.subcommand() {
        ("lss", Some(_)) => {
            let items = list_skillnuggets(&conn).unwrap();
            println!("{} skill nuggets found:", items.len());
            for i in items {
                println!("{:?}", i);
            };
        },
        ("lsq", Some(_)) => {
            let items = list_questions(&conn).unwrap();
            println!("{} questions found:", items.len());
            for i in items {
                println!("{:?}", i);
            };
        },
        _ => {
            unreachable!(); // clap should exit before reaching here if none of the subcommands are entered.
        },
    }
}
