use std::fmt::Debug;

pub use anyhow::{Result, anyhow, Context};

pub trait ResultExt: Sized {
    fn handle<E>(self: Self) -> std::result::Result<E, Self>
        where E: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;
}

impl<T> ResultExt for Result<T, anyhow::Error> {
    fn handle<E>(self: Self) -> Result<E, Self>
        where E: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static {
        match self {
            Ok(ok) => Err(Ok(ok)),
            Err(err) => match err.downcast::<E>() {
                Ok(err_to_handle) => Ok(err_to_handle),
                Err(unknown_err) => Err(Err(unknown_err)),
            }
        }
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
#[error("FileAlreadyExists")]
pub struct FileAlreadyExists{pub hash: Vec<u8>}
#[derive(Error, Debug)]
#[error("NoSuchUser")]
pub struct NoSuchUser{pub username: String}
#[derive(Error, Debug)]
#[error("PasswordDoesntMatch")]
pub struct PasswordDoesntMatch;
#[derive(Error, Debug)]
#[error("RateLimitExceeded")]
pub struct RateLimitExceeded;
#[derive(Error, Debug)]
#[error("NoneResultYo")]
pub struct NoneResultYo;

/*
error_chain! {
        foreign_links {
            ParseBoolError(::std::str::ParseBoolError);
            VarError(::std::env::VarError);
            ParseIntError(::std::num::ParseIntError);
            ParseFloatError(::std::num::ParseFloatError);
            StdIoError(::std::io::Error);
            DieselError(::diesel::result::Error);
            DieselMigrationError(::diesel::migrations::RunMigrationsError);
            FmtError(::std::fmt::Error);
            R2D2Error(::r2d2::GetTimeout);
            DataEncodingError(::data_encoding::decode::Error);
            ChronoParseError(::chrono::ParseError);
        }
        errors {
            InvalidInput {
                description("Provided input is invalid.")
                display("Provided input is invalid.")
            }
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
            BadSessId {
                description("Malformed session ID!")
                display("Malformed session ID!")
            }
            NoSuchSess {
                description("Session doesn't exist!")
                display("Session doesn't exist!")
            }
            FormParseError {
                description("Can't parse the HTTP form!")
                display("Can't parse the HTTP form!")
            }
            FileNotFound {
                description("Can't find that file!")
                display("Can't find that file!")
            }
            DatabaseOdd(reason: &'static str) {
                description(
                    "There's something wrong with the contents of the DB vs. how it should be!"
                )
                display(
                    "There's something wrong with the contents of the DB vs. how it should be! {}"
                , reason)
            }
            AccessDenied {
                description("Access denied")
                display("Access denied")
            }
            NoneResult {
                description("Option::None")
                display("Option::None")
            }
            RateLimitExceeded {
                description("RateLimit exceeded")
                display("RateLimit exceeded")
            }
        }
    }
*/