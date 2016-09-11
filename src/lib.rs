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
extern crate data_encoding;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod password;
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
            PasswordDoesntMatch {
                description("Password doesn't match")
                display("Password doesn't match.")
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


pub fn add_user(conn : &PgConnection, email : &str, password : &str) -> Result<()> {
    use schema::{users, passwords};
    use models::{NewUser, NewPassword, Password};

    if email.len() > 254 { return Err(ErrorKind::EmailAddressTooLong.into()) };
    if !email.contains("@") { return Err(ErrorKind::EmailAddressNotValid.into()) };

    let pw = password::set_password(password).chain_err(|| "Setting password didn't succeed!")?;

    let pw_row : NewPassword = pw.into();
    let db_pw : Password = diesel::insert(&pw_row)
        .into(passwords::table)
        .get_result(conn)
        .chain_err(|| "Couldn't insert the new password into database!")?;

    let new_user = NewUser {
        email : email,
        password : db_pw.id,
    };

    diesel::insert(&new_user).into(users::table).execute(conn).chain_err(|| "Couldn't create a new user!")?;
    Ok(())
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

