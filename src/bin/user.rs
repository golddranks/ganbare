extern crate ganbare;
extern crate diesel;

#[macro_use]
extern crate clap;
extern crate rpassword;

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
                let email = args.value_of("email").unwrap();
                println!("Removing user with e-mail {}", email);

                let user = match get_user_by_email(&email, &conn) {
                    Ok(u) => u,
                    Err(e) => { println!("Error: {}", e); return; },
                };
                match remove_user(&conn, email) {
                    Ok(_) => { println!("Success! User removed. Removed user: {:?}", user); },
                    Err(e) => { println!("Error: {}", e); return; },
                };
            },
            ("add", Some(args)) => {
                use ganbare::errors::ErrorKind::NoSuchUser;
                let email = args.value_of("email").unwrap();
                match get_user_by_email(&email, &conn) {
                    Err(Error(kind, _)) => match kind {
                        NoSuchUser(email) => println!("Adding a user with email {}", email),
                        _ => { println!("Error: {:?}", kind); return; },
                    },
                    Ok(_) => { println!("Error: User already exists!"); return; },
                }
                println!("Enter a password:");
                let password = match rpassword::read_password() {
                    Err(_) => { println!("Error: couldn't read the password from keyboard."); return; },
                    Ok(pw) => pw,
                };
                match add_user(&conn, email, &password) {
                    Ok(u) => println!("Added user successfully: {:?}", u),
                    Err(err_chain) => for err in err_chain.iter() { println!("Error: {}", err) },
                }
            },
         _ => {
            unreachable!(); // clap should exit before reaching here if none of the subcommands are entered.
         },
        }
    
}
