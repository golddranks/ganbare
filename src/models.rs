use super::schema::*;

use diesel::ExpressionMethods;
use chrono::{DateTime, UTC};

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[has_many(passwords, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(sessions, foreign_key = "user_id")]
#[derive(Identifiable, Queryable, Debug, Associations)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub joined: DateTime<UTC>,
}


#[derive(Identifiable, Queryable, Debug, Insertable, Associations)]
#[belongs_to(User, foreign_key = "id")]
#[table_name="passwords"]
pub struct Password {
    pub id: i32,
    pub password_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub initial_rounds: i16,
    pub extra_rounds: i16,
}

/*
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
*/

#[derive(Debug, Insertable)]
#[table_name="sessions"]
pub struct NewSession<'a> {
    pub sess_id: &'a [u8],
    pub user_id: i32,
    pub last_ip: Vec<u8>,
}


#[derive(AsChangeset)]
#[table_name="sessions"]
pub struct RefreshSession<'a> {
    pub sess_id: &'a [u8],
    pub last_ip: Vec<u8>,
    pub last_seen: DateTime<UTC>,
}

#[derive(Identifiable, Queryable, Debug, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
pub struct Session {
    pub id: i32,
    pub sess_id: Vec<u8>,
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: Vec<u8>,
}
/*
BelongsTo! {
    (User, foreign_key = user_id)
    #[table_name(sessions)]
    pub struct Session {
        pub id: i32,
        pub sess_id: [u8; 32],
        pub user_id: i32,
        pub started: DateTime<UTC>,
        pub last_seen: DateTime<UTC>,
        pub last_ip: Vec<u8>,
}
}
*/

#[derive(Queryable, Debug)]
pub struct PendingEmailConfirm {
    pub secret: String,
    pub email: String,
    pub added: DateTime<UTC>,
}


#[derive(Insertable)]
#[table_name="pending_email_confirms"]
pub struct NewPendingEmailConfirm<'a> {
    pub secret: &'a str,
    pub email: &'a str,
}
