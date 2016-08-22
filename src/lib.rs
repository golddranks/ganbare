#![recursion_limit = "1024"]
#![feature(custom_derive, question_mark, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate error_chain;

extern crate dotenv;
extern crate crypto;
extern crate chrono;

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
                description("No such user exists.")
                display("No user with e-mail address {} exists.", email)
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

pub fn add_user(conn : &PgConnection, email : &str) -> Result<usize> {
    use schema::users;
    use models::NewUser;

    let new_user = NewUser {
        email : email,
        password_hash : b"",
        salt : b"",
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

