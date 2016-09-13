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
use models::{User, Password};
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
                display("A valid password must be at least 8 characters (bytes).")
            }
            PasswordTooLong {
                description("Password too long")
                display("A valid password must be at maximum 1024 characters (bytes).")
            }
            PasswordDoesntMatch {
                description("Password doesn't match")
                display("Password doesn't match.")
            }
            AuthError {
                description("Can't authenticate user")
                display("Username (= e-mail) or password doesn't match.")
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

pub fn get_user_by_email(conn : &PgConnection, user_email : &str) -> Result<models::User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    users
        .filter(email.eq(user_email))
        .first(conn)
        .map_err(|e| match e {
                e @ NotFound => Err::<(), diesel::result::Error>(e).chain_err(|| ErrorKind::NoSuchUser(user_email.into())).unwrap_err(),
                e => Err::<(), diesel::result::Error>(e).chain_err(|| "Error when trying to retrieve user!").unwrap_err(),
        })
}

fn get_user_pass_by_email(conn : &PgConnection, user_email : &str) -> Result<(User, Password)> {
    use schema::users;
    use schema::passwords;
    use diesel::result::Error::NotFound;

    users::table
        .inner_join(passwords::table)
        .filter(users::email.eq(user_email))
        .first(&*conn)
        .map_err(|e| match e {
                e @ NotFound => Err::<(), diesel::result::Error>(e).chain_err(|| ErrorKind::NoSuchUser(user_email.into())).unwrap_err(),
                e => Err::<(), diesel::result::Error>(e).chain_err(|| "Error when trying to retrieve user!").unwrap_err(),
        })
}

pub fn add_user(conn : &PgConnection, email : &str, password : &str) -> Result<User> {
    use schema::{users, passwords};
    use models::{NewUser, User};

    if email.len() > 254 { return Err(ErrorKind::EmailAddressTooLong.into()) };
    if !email.contains("@") { return Err(ErrorKind::EmailAddressNotValid.into()) };

    let pw = password::set_password(password).chain_err(|| "Setting password didn't succeed!")?;

    let new_user = NewUser {
        email : email,
    };

    let user : User = diesel::insert(&new_user)
        .into(users::table)
        .get_result(conn)
        .chain_err(|| "Couldn't create a new user!")?;

    diesel::insert(&pw.into_db(user.id))
        .into(passwords::table)
        .execute(conn)
        .chain_err(|| "Couldn't insert the new password into database!")?;

    Ok(user)
}

pub fn remove_user(conn : &PgConnection, rm_email : &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    diesel::delete(users.filter(email.eq(rm_email)))
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => Err::<(), diesel::result::Error>(e).chain_err(|| ErrorKind::NoSuchUser(rm_email.into())).unwrap_err(),
                e => Err::<(), diesel::result::Error>(e).chain_err(|| "Couldn't remove the user!").unwrap_err(),
        })
}

pub fn auth_user(conn : &PgConnection, email : &str, plaintext_pw : &str) -> Result<User> {
    let (user, hashed_pw_from_db) = get_user_pass_by_email(conn, email)
                                .map_err::<Error, _>(|_| ErrorKind::AuthError.into())?;
    let _ = password::check_password(plaintext_pw, hashed_pw_from_db.into())
                                .map_err::<Error, _>(|_| ErrorKind::AuthError.into())?;
    Ok(user)
}
    
