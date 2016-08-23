#![recursion_limit = "1024"]
#![feature(custom_derive, question_mark, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros, binary_macros)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate error_chain;

extern crate dotenv;
extern crate crypto;
extern crate chrono;
extern crate rand;
extern crate rustc_serialize;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod errors {
    error_chain! {
        errors {
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
                display("A valid password must be at least 8 characters.")
            }
        }
    }
}

use errors::*;

pub fn establish_connection() -> Result<PgConnection> {
    dotenv().ok();
    let database_url = env::var("GANBARE_DATABASE_URL")
        .chain_err(|| "GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")?;
    PgConnection::establish(&database_url)
        .chain_err(|| "Error connecting to database!")
}

pub fn set_password(password : &str) -> Result<(/*hash : */[u8; 24], /*salt : */[u8; 16], /*rounds : */i16)> {
    use rand::{OsRng, Rng};
    use rustc_serialize::base64::FromBase64;
    use crypto::bcrypt::bcrypt;

    if password.len() < 8 { return Err(ErrorKind::PasswordTooShort.into()) };

    let mut salt = [0_u8; 16];
    OsRng::new().chain_err(|| "Unable to connect to the system random number generator!")?.fill_bytes(&mut salt);
    let salt = salt;

    dotenv().ok();
    let build_pepper = base64!(dotenv!("GANBARE_BUILDTIME_PEPPER"));
    let run_pepper = env::var("GANBARE_RUNTIME_PEPPER").chain_err(|| "Environmental variable GANBARE_RUNTIME_PEPPER must be set!")?
        .from_base64().chain_err(|| "Environmental variable GANBARE_RUNTIME_PEPPER isn't valid Base64!")?;

// TODO Apply pepper
    let peppered_pw = password.as_bytes();// ^ build_pepper ^ run_pepper;

// TODO Calculate rounds
    let mut hash = [0_u8; 24];
    bcrypt(10, &salt, peppered_pw, &mut hash);

    Ok((hash, salt, 0))
}

pub fn add_user(conn : &PgConnection, email : &str, password : &str) -> Result<usize> {
    use schema::users;
    use models::NewUser;

    if email.len() > 254 { return Err(ErrorKind::EmailAddressTooLong.into()) };
    if !email.contains("@") { return Err(ErrorKind::EmailAddressNotValid.into()) };

    let (password_hash, salt, rounds) = set_password(password).chain_err(|| "Setting password didn't succeed!")?;

    let new_user = NewUser {
        email : email,
        password_hash : &password_hash,
        salt : &salt,
        rounds : rounds,
    };

    diesel::insert(&new_user).into(users::table).execute(conn).chain_err(|| "Couldn't create a new user!")
}

pub fn remove_user(conn : &PgConnection, rm_email : &str) -> Result<usize> {
    use schema::users::dsl::*;

    match diesel::delete(users.filter(email.eq(rm_email))).execute(conn) {
        Ok(0) => Err(ErrorKind::NoSuchUser(rm_email.into()).into()),
        Ok(1) => Ok(1),
        Ok(_) => unreachable!(), // Two or more users with the same e-mail address can't exist, by the "unique" constraint in DB schema.
        Err(e) => Err(e).chain_err(|| "Couldn't remove the user!"),
    }
    

}

