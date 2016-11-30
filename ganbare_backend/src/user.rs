   use super::*;

/* TODO FIXME this can be a full-blown typed group system some day 
enum Group {
    Admins,
    Editors,
    Betatesters,
    Subjects,
    InputGroup,
    OutputGroup,
    ShowAccent,
    Other(String),
}*/

pub fn get_user_by_email(conn : &PgConnection, user_email : &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    users
        .filter(email.eq(user_email))
        .first(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(user_email.into())),
                e => e.caused_err(|| "Error when trying to retrieve user!"),
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
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(user_email.into())),
                e => e.caused_err(|| "Error when trying to retrieve user!"),
        })
}


pub fn auth_user(conn : &PgConnection, email : &str, plaintext_pw : &str, pepper: &[u8]) -> Result<Option<User>> {
    let (user, hashed_pw_from_db) = match get_user_pass_by_email(conn, email) {
        Err(err) => match err.kind() {
            &ErrorKind::NoSuchUser(_) => return Ok(None),
            _ => Err(err),
        },
        ok => ok,
    }?;

    match password::check_password(plaintext_pw, hashed_pw_from_db.into(), pepper) {
        Err(err) => match err.kind() {
            &ErrorKind::PasswordDoesntMatch => return Ok(None),
            _ => Err(err),
        },
        ok => ok,
    }?;
    
    Ok(Some(user))
}


pub fn add_user(conn : &PgConnection, email : &str, password : &str, pepper: &[u8]) -> Result<User> {
    use schema::{users, passwords, user_metrics};

    if email.len() > 254 { return Err(ErrorKind::EmailAddressTooLong.into()) };
    if !email.contains("@") { return Err(ErrorKind::EmailAddressNotValid.into()) };

    let pw = password::set_password(password, pepper)?;

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

    diesel::insert(&NewUserMetrics{ id: user.id })
        .into(user_metrics::table)
        .execute(conn)
        .chain_err(|| "Couldn't insert the new password into user metrics!")?;

    info!("Created a new user, with email {:?}.", email);
    Ok(user)
}

pub fn set_password(conn : &PgConnection, user_email : &str, password: &str, pepper: &[u8]) -> Result<User> {
    use schema::{users, passwords};

    let (u, p) : (User, Option<Password>) = users::table
        .left_outer_join(passwords::table)
        .filter(users::email.eq(user_email))
        .first(&*conn)
        .map_err(|e| e.caused_err(|| "Error when trying to retrieve user!"))?;
    if p.is_none() {

        let pw = password::set_password(password, pepper).chain_err(|| "Setting password didn't succeed!")?;

        diesel::insert(&pw.into_db(u.id))
            .into(passwords::table)
            .execute(conn)
            .chain_err(|| "Couldn't insert the new password into database!")?;

        Ok(u)
    } else {
        Err("Password already set!".into())
    }
}

pub fn remove_user(conn : &PgConnection, rm_email : &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    diesel::delete(users.filter(email.eq(rm_email)))
        .get_result(conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::NoSuchUser(rm_email.into())),
                e => e.caused_err(|| "Couldn't remove the user!"),
        })
}

pub fn change_password(conn : &PgConnection, user_id : i32, new_password : &str, pepper: &[u8]) -> Result<()> {

    let pw = password::set_password(new_password, pepper).chain_err(|| "Setting password didn't succeed!")?;

    let _ : models::Password = pw.into_db(user_id).save_changes(conn)?;

    Ok(())
}


pub fn join_user_group_by_id(conn: &PgConnection, user: &User, group_id: i32) -> Result<()> {
    use schema::{user_groups, group_memberships};

    let group: UserGroup = user_groups::table
        .filter(user_groups::id.eq(group_id))
        .first(conn)?;

    diesel::insert(&GroupMembership{ user_id: user.id, group_id: group.id, anonymous: false})
                .into(group_memberships::table)
                .execute(conn)?;
    Ok(())
}

pub fn join_user_group_by_name(conn: &PgConnection, user: &User, group_name: &str) -> Result<()> {
    use schema::{user_groups, group_memberships};

    let group: UserGroup = user_groups::table
        .filter(user_groups::group_name.eq(group_name))
        .first(conn)?;

    diesel::insert(&GroupMembership{ user_id: user.id, group_id: group.id, anonymous: false})
                .into(group_memberships::table)
                .execute(conn)?;
    Ok(())
}

pub fn check_user_group(conn : &PgConnection, user: &User, group_name: &str )  -> Result<bool> {
    use schema::{user_groups, group_memberships};

    if group_name == "" { return Ok(true) };

    let exists : Option<(UserGroup, GroupMembership)> = user_groups::table
        .inner_join(group_memberships::table)
        .filter(group_memberships::user_id.eq(user.id))
        .filter(user_groups::group_name.eq(group_name))
        .get_result(&*conn)
        .optional()
        .chain_err(|| "DB error")?;

    Ok(exists.is_some())
}

pub fn get_group(conn : &PgConnection, group_name: &str )  -> Result<Option<UserGroup>> {
    use schema::user_groups;

    let group : Option<(UserGroup)> = user_groups::table
        .filter(user_groups::group_name.eq(group_name))
        .get_result(&*conn)
        .optional()?;

    Ok(group)
}
