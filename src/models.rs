use super::schema::users;
use super::schema::passwords;

use chrono::{DateTime, UTC};

#[insertable_into(users)]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub joined: DateTime<UTC>,
}

#[insertable_into(passwords)]
#[derive(Queryable, Debug)]
pub struct Password {
    pub id: i32,
    pub salt: Vec<u8>,
    pub password_hash: Vec<u8>,
    pub initial_rounds: i16,
    pub extra_rounds: i16,
}
