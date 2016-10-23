extern crate ganbare;
extern crate diesel;

#[macro_use]
extern crate clap;
extern crate rpassword;
extern crate lettre;
extern crate dotenv;
extern crate handlebars;
extern crate rustc_serialize;

use rustc_serialize::json::{ToJson, Json};
use handlebars::Handlebars;
use dotenv::dotenv;
use std::env;
use std::collections::BTreeMap;
use std::io::Read;
use ganbare::*;
use ganbare::models::User;
use diesel::prelude::*;
use ganbare::errors::*;

use lettre::transport::smtp::response::Response as EmailResponse;
use lettre::transport::smtp::SmtpTransportBuilder;
use lettre::transport::EmailTransport;

pub fn list_users(conn : &PgConnection) -> Result<Vec<User>> {
    use ganbare::schema::users::dsl::*;
 
    users.load::<User>(conn).chain_err(|| "Can't load users")
}


fn send_email_confirmation(email : &str, secret : &str) -> Result<EmailResponse> {
    use lettre::email::EmailBuilder;

    dotenv().ok();
    let site_domain = env::var("GANBARE_EMAIL_DOMAIN").unwrap_or_else(|_| "".into());
    let mut email_template = String::new();
    std::fs::File::open("templates/email_confirm_email.html")
        .chain_err(|| "Can't find templates/email_confirm_email.txt!")?
        .read_to_string(&mut email_template)?;
    let handlebars = Handlebars::new();

    struct EmailData<'a> { secret: &'a str };

    impl<'a> ToJson for EmailData<'a> {
        fn to_json(&self) -> Json {
            let mut m: BTreeMap<String, Json> = BTreeMap::new();
            m.insert("secret".to_string(), self.secret.to_json());
            m.to_json()
        }
    }

    let data = EmailData { secret: secret };

    let email = EmailBuilder::new()
        .to(email)
        .from(format!("noreply@{}", site_domain).as_ref())
        .subject("[akusento.ganba.re] Vahvista osoitteesi")
        .html(handlebars.template_render(email_template.as_ref(), &data)
            .chain_err(|| "Handlebars template render error!")?
            .as_ref())
        .build().expect("Building email shouldn't fail.");

    let mut mailer = SmtpTransportBuilder::new(("mail.inet.fi", 25))
        .chain_err(|| "Couldn't setup the email transport!")?
        .build();
    mailer.send(email)
        .chain_err(|| "Couldn't send email!")
}

fn main() {
    use clap::*;

    let matches = App::new("ganba.re user control")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .subcommand(SubCommand::with_name("ls").about("List all users"))
        .subcommand(SubCommand::with_name("rm").about("Remove user").arg(Arg::with_name("email").required(true)))
        .subcommand(SubCommand::with_name("add").about("Add a new user").arg(Arg::with_name("email").required(true)))
        .subcommand(SubCommand::with_name("login").about("Login").arg(Arg::with_name("email").required(true)))
        .get_matches();
    let conn = db_connect().unwrap();
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
            match remove_user(&conn, email) {
                Ok(user) => { println!("Success! User removed. Removed user: {:?}", user); },
                Err(e) => { println!("Error: {}", e); return; },
            };
        },
        ("add", Some(args)) => {
            use ganbare::errors::ErrorKind::NoSuchUser;
            let email = args.value_of("email").unwrap();
            match get_user_by_email(&conn, &email) {
                Err(Error(kind, _)) => match kind {
                    NoSuchUser(email) => println!("Adding a user with email {}", email),
                    _ => { println!("Error: {:?}", kind); return; },
                },
                Ok(_) => { println!("Error: User already exists!"); return; },
            }
            let secret = match ganbare::add_pending_email_confirm(&conn, email) {
                Ok(secret) => secret,
                Err(e) => { println!("Error: {:?}", e); return; }
            };
            match send_email_confirmation(email, secret.as_ref()) {
                Ok(u) => println!("Sent an email confirmation! {:?}", u),
                Err(err_chain) => for err in err_chain.iter() { println!("Error: {}\nCause: {:?}", err, err.cause ()) },
            }
        },
        ("add_with_password", Some(args)) => {
            use ganbare::errors::ErrorKind::NoSuchUser;
            let email = args.value_of("email").unwrap();
            match get_user_by_email(&conn, &email) {
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
        ("login", Some(args)) => {
            let email = args.value_of("email").unwrap();
            println!("Enter a password:");
            let password = match rpassword::read_password() {
                Err(_) => { println!("Error: couldn't read the password from keyboard."); return; },
                Ok(pw) => pw,
            };
            match auth_user(&conn, email, &password) {
                Ok(u) => println!("Logged in successfully: {:?}", u),
                Err(err_chain) => for err in err_chain.iter() { println!("Error: {}", err) },
            }
        },
        _ => {
            unreachable!(); // clap should exit before reaching here if none of the subcommands are entered.
        },
    }
}
