extern crate ganbare;
extern crate diesel;

#[macro_use]
extern crate clap;

use ganbare::*;
use ganbare::models::User;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use ganbare::errors::*;


pub fn list_users(conn : &PgConnection) -> Result<Vec<User>> {
    use ganbare::schema::users::dsl::*;
 
    users.load::<User>(conn).chain_err(|| "Can't load users")
}

fn main() {
    use clap::*;

    let matches = App::new("ganba.re user control")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .subcommand(SubCommand::with_name("ls").about("List all users"))
        .subcommand(SubCommand::with_name("rm").about("Remove user").arg(Arg::with_name("email").required(true)))
        .subcommand(SubCommand::with_name("add").about("Add a new user").arg(Arg::with_name("email").required(true)))
        .get_matches();
        let conn = establish_connection().unwrap();
        match matches.subcommand() {
            ("ls", Some(_)) => {
                let users = list_users(&conn).unwrap();
                println!("{} users found:", users.len());
                for user in users {
                    println!("{:?}", user);
                };
            },
            ("rm", Some(args)) => {
                println!("Removing user with email {}", args.value_of("email").unwrap());
            },
            ("add", Some(args)) => {
                let email = args.value_of("email").unwrap();
                println!("Adding a user with email {}", email);
                add_user(&conn, email).unwrap();
            },
         _ => {
            unreachable!(); // clap should exit before this if none of the subcommands are entered.
         },
        }
    
}
