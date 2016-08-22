#![recursion_limit = "1024"]
#![feature(custom_derive, question_mark, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate error_chain;

extern crate dotenv;
extern crate crypto;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;


mod schema;
mod models;

mod errors {
    error_chain! { }
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
        password_hash : "",
        salt : "",
    };

    diesel::insert(&new_user).into(users::table).execute(conn).chain_err(|| "Couldn't create a new user!")
}
