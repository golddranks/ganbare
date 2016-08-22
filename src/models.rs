

use super::schema::users;

#[insertable_into(users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub salt: &'a str,
}
