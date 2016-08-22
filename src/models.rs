use super::schema::users;

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
    pub joined: super::diesel::data_types::PgDate,
    pub salt: Vec<u8>,
    pub password_hash: Vec<u8>,
    pub rounds: i16,
}
