extern crate ganbare;

#[macro_use]
extern crate clap;

use ganbare::*;
use clap::*;

fn main() {
    let matches = App::new("ganba.re user control")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .subcommand(SubCommand::with_name("ls").about("List all users"))
        .subcommand(SubCommand::with_name("rm").about("Remove user").arg(Arg::with_name("email").required(true)))
        .subcommand(SubCommand::with_name("add").about("Add a new user").arg(Arg::with_name("email").required(true)))
        .get_matches();
        match matches.subcommand() {
        ("ls", None) => {

        },
        ("rm", Some(args)) => {
            println!("Removing user with email {}", args.value_of("email").unwrap());
        },
        ("add", Some(args)) => {
            println!("Adding a user with email {}", args.value_of("email").unwrap());
            let conn = establish_connection().unwrap();
            add_user(&conn, "").unwrap();
        },
        _ => {},
        }
    
}
