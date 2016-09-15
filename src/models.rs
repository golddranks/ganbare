use super::schema::*;

use chrono::{DateTime, UTC};

#[insertable_into(users)]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[has_many(passwords, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(sessions, foreign_key = "user_id")]
#[derive(Identifiable, Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub joined: DateTime<UTC>,
}


#[insertable_into(passwords)]
#[derive(Identifiable, Queryable, Debug)]
pub struct Password {
    pub id: i32,
    pub password_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub initial_rounds: i16,
    pub extra_rounds: i16,
}


BelongsTo! {
    (User, foreign_key = id)
    #[table_name(passwords)]
    pub struct Password {
        pub id: i32,
        pub password_hash: Vec<u8>,
        pub salt: Vec<u8>,
        pub initial_rounds: i16,
        pub extra_rounds: i16,
    }
}

#[insertable_into(sessions)]
#[derive(Debug)]
pub struct NewSession {
    pub id: Vec<u8>,
    pub user_id: i32,
    pub last_ip: Vec<u8>,
}

#[derive(Identifiable, Queryable, Debug)]
pub struct Session {
    pub id: Vec<u8>,
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: Vec<u8>,
}

BelongsTo! {
    (User, foreign_key = user_id)
    #[table_name(sessions)]
    pub struct Session {
        pub id: [u8; 32],
        pub user_id: i32,
        pub started: DateTime<UTC>,
        pub last_seen: DateTime<UTC>,
        pub last_ip: Vec<u8>,
}
}
