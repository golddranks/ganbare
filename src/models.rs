use super::schema::users;
use chrono::{DateTime, UTC};

#[insertable_into(users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a [u8],
    pub salt: &'a [u8],
}

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub joined: DateTime<UTC>,
    pub salt: Vec<u8>,
    pub password_hash: Vec<u8>,
    pub rounds: i16,
}
