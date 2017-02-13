use super::*;
use std::time::Instant;
use data_encoding;
use chrono::Duration;

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

pub fn get_user_by_email(conn: &Connection, user_email: &str) -> Result<Option<User>> {
    use schema::users::dsl::*;

    Ok(users.filter(sql::lower(email).eq(sql::lower(user_email)))
        .first(&**conn)
        .optional()?)
}

fn get_user_pass_by_email(conn: &Connection, user_email: &str) -> Result<(User, Password)> {
    use schema::users;
    use schema::passwords;
    use diesel::result::Error::NotFound;

    users::table.inner_join(passwords::table)
        .filter(sql::lower(users::email).eq(sql::lower(user_email)))
        .first(&**conn)
        .or_else(|e| match e {
            e @ NotFound => Err(e).chain_err(|| ErrorKind::NoSuchUser(user_email.into())),
            e => Err(e).chain_err(|| "Error when trying to retrieve user!"),
        })
}


pub fn auth_user(conn: &Connection,
                 email: &str,
                 plaintext_pw: &str,
                 pepper: &[u8])
                 -> Result<Option<User>> {
    let (user, hashed_pw_from_db) = match get_user_pass_by_email(conn, email) {
        Err(err) => {
            match *err.kind() {
                ErrorKind::NoSuchUser(_) => return Ok(None),
                _ => Err(err),
            }
        }
        ok => ok,
    }?;

    let time_before = Instant::now();
    match password::check_password(plaintext_pw, hashed_pw_from_db.into(), pepper) {
        Err(err) => {
            match *err.kind() {
                ErrorKind::PasswordDoesntMatch => return Ok(None),
                _ => Err(err),
            }
        }
        ok => ok,
    }?;
    let time_after = Instant::now();
    info!("Checked password. Time spent: {} ms",
          (time_after - time_before).as_secs() * 1000 +
          (time_after - time_before).subsec_nanos() as u64 / 1_000_000);

    Ok(Some(user))
}


pub fn add_user(conn: &Connection, email: &str, password: &str, pepper: &[u8]) -> Result<User> {
    use schema::{users, passwords, user_metrics, user_stats};

    if email.len() > 254 {
        return Err(ErrorKind::EmailAddressTooLong.into());
    };
    if !email.contains('@') {
        return Err(ErrorKind::EmailAddressNotValid.into());
    };

    let pw = password::set_password(password, pepper)?;

    let new_user = NewUser { email: email };

    let user: User = diesel::insert(&new_user).into(users::table)
        .get_result(&**conn)?;

    diesel::insert(&pw.into_db(user.id)).into(passwords::table)
        .execute(&**conn)?;

    diesel::insert(&NewUserMetrics { id: user.id }).into(user_metrics::table)
        .execute(&**conn)?;

    diesel::insert(&NewUserStats { id: user.id }).into(user_stats::table)
        .execute(&**conn)?;

    info!("Created a new user, with email {:?}.", email);
    Ok(user)
}

pub fn set_password(conn: &Connection,
                    user_email: &str,
                    password: &str,
                    pepper: &[u8])
                    -> Result<User> {
    use schema::{users, passwords};

    let (u, p): (User, Option<Password>) = users::table.left_outer_join(passwords::table)
        .filter(sql::lower(users::email).eq(sql::lower(user_email)))
        .first(&**conn)
        .chain_err(|| "Error when trying to retrieve user!")?;
    if p.is_none() {

        let pw = password::set_password(password, pepper)
            .chain_err(|| "Setting password didn't succeed!")?;

        diesel::insert(&pw.into_db(u.id)).into(passwords::table)
            .execute(&**conn)
            .chain_err(|| "Couldn't insert the new password into database!")?;

        Ok(u)
    } else {
        Err("Password already set!".into())
    }
}

pub fn check_password_reset(conn: &Connection,
                            secret: &str)
                            -> Result<Option<(ResetEmailSecrets, User)>> {
    use schema::{reset_email_secrets, users};

    let confirm: Option<(ResetEmailSecrets, User)> =
        reset_email_secrets::table.inner_join(users::table)
            .filter(reset_email_secrets::secret.eq(secret))
            .first(&**conn)
            .optional()?;

    Ok(match confirm {
        Some((c, u)) => {
            if c.added < chrono::UTC::now() - chrono::Duration::days(1) {
                diesel::delete(
                    reset_email_secrets::table
                        .filter(reset_email_secrets::user_id.eq(c.user_id))
                ).execute(&**conn)?;
                None
            } else {
                Some((c, u))
            }
        }
        None => None,
    })
}

pub fn invalidate_password_reset(conn: &Connection, secret: &ResetEmailSecrets) -> Result<()> {
    use schema::reset_email_secrets;

    diesel::delete(
        reset_email_secrets::table
            .filter(reset_email_secrets::user_id.eq(secret.user_id))
    ).execute(&**conn)?;
    Ok(())
}

pub fn send_pw_change_email(conn: &Connection, email: &str) -> Result<ResetEmailSecrets> {
    use schema::{users, reset_email_secrets};

    let earlier_email: Option<(ResetEmailSecrets, User)> =
        reset_email_secrets::table.inner_join(users::table)
            .filter(sql::lower(users::email).eq(sql::lower(email)))
            .order(reset_email_secrets::added.desc())
            .get_result(&**conn)
            .optional()?;

    if let Some((secret, user)) = earlier_email {
        if secret.added > chrono::UTC::now() - chrono::Duration::days(1) {
            return Err(ErrorKind::RateLimitExceeded.into()); // Flood filter
        } else {
            // Possible to send a new request; delete/invalidate the earlier ones:
            diesel::delete(
                reset_email_secrets::table
                    .filter(reset_email_secrets::user_id.eq(user.id))
            ).execute(&**conn)?;
        }
    }

    let user: Option<User> = users::table.filter(sql::lower(users::email).eq(sql::lower(&email)))
        .get_result(&**conn)
        .optional()?;

    let user = match user {
        Some(user) => user,
        None => return Err(ErrorKind::NoSuchUser(email.to_string()).into()),
    };

    let secret = data_encoding::base64url::encode(&session::fresh_token()?[..]);

    let result = diesel::insert(&ResetEmailSecrets {
            secret: secret,
            user_id: user.id,
            email: user.email.expect("We just found this user by the email address!"),
            added: chrono::UTC::now(),
        }).into(reset_email_secrets::table)
        .get_result(&**conn)?;

    Ok(result)
}

pub fn remove_user_by_email(conn: &Connection, rm_email: &str) -> Result<User> {
    use schema::users::dsl::*;
    use diesel::result::Error::NotFound;

    diesel::delete(users.filter(sql::lower(email).eq(sql::lower(rm_email))))
        .get_result(&**conn)
        .map_err(|e| match e {
            e @ NotFound => Error::from_err(e,  ErrorKind::NoSuchUser(rm_email.into())),
            e => Error::from_err(e, "Couldn't remove the user!".into()),
        })
}

pub fn deactivate_user(conn: &Connection, id: i32) -> Result<Option<User>> {
    use schema::users;

    let user = match users::table.filter(users::id.eq(id))
        .get_result::<User>(&**conn)
        .optional()? {
        Some(u) => u,
        None => return Ok(None),
    };

    let no_email: Option<String> = None;

    diesel::delete(schema::passwords::table.filter(schema::passwords::id.eq(id))).execute(&**conn)?;
    diesel::update(users::table.filter(users::id.eq(id))).set(users::email.eq(no_email))
        .execute(&**conn)?;

    Ok(Some(user))
}

pub fn remove_user_completely(conn: &Connection, id: i32) -> Result<Option<User>> {
    use schema::users;

    let user = match users::table.filter(users::id.eq(id))
        .get_result::<User>(&**conn)
        .optional()? {
        Some(u) => u,
        None => return Ok(None),
    };

    macro_rules! delete {
        ($table:ident, $field:ident) => {
            diesel::delete(
                schema::$table::table.filter(schema::$table::$field.eq(id))
            ).execute(&**conn)?;
        }
    }

    delete!(passwords, id);
    delete!(user_metrics, id);
    delete!(user_stats, id);
    delete!(sessions, user_id);
    delete!(skill_data, user_id);
    delete!(event_experiences, user_id);
    delete!(group_memberships, user_id);
    delete!(anon_aliases, user_id);
    delete!(pending_items, user_id);
    delete!(due_items, user_id);
    delete!(users, id);

    Ok(Some(user))
}

pub fn change_password(conn: &Connection,
                       user_id: i32,
                       new_password: &str,
                       pepper: &[u8])
                       -> Result<()> {

    let pw = password::set_password(new_password, pepper)
        .chain_err(|| "Setting password didn't succeed!")?;

    let _: models::Password = pw.into_db(user_id).save_changes(&**conn)?;

    Ok(())
}


pub fn join_user_group_by_id(conn: &Connection,
                             user_id: i32,
                             group_id: i32)
                             -> Result<GroupMembership> {
    use schema::group_memberships;

    let membership: GroupMembership = diesel::insert(&GroupMembership {
            user_id: user_id,
            group_id: group_id,
            anonymous: false,
        }).into(group_memberships::table)
        .get_result(&**conn)?;
    Ok(membership)
}


pub fn remove_user_group_by_id(conn: &Connection, user_id: i32, group_id: i32) -> Result<bool> {
    use schema::group_memberships;

    let count =
        diesel::delete(group_memberships::table.filter(group_memberships::user_id.eq(user_id)
                .and(group_memberships::group_id.eq(group_id)))).execute(&**conn)?;

    Ok(count == 1)
}

pub fn group_size(conn: &Connection, group_name: &str) -> Result<i64> {
    use schema::{group_memberships, user_groups};

    let group: UserGroup = user_groups::table.filter(user_groups::group_name.eq(group_name))
        .get_result(&**conn)?;

    let count = group_memberships::table.filter(group_memberships::group_id.eq(group.id))
        .count()
        .get_result(&**conn)?;

    Ok(count)
}

pub fn group_intersection_size(conn: &Connection,
                               group_name_a: &str,
                               group_name_b: &str)
                               -> Result<i64> {
    use schema::{users, group_memberships, user_groups};
    use diesel::expression::dsl::any;

    let group_a: UserGroup = user_groups::table.filter(user_groups::group_name.eq(group_name_a))
        .get_result(&**conn)?;
    let group_a_members =
        group_memberships::table.filter(group_memberships::group_id.eq(group_a.id))
            .select(group_memberships::user_id);

    let group_b: UserGroup = user_groups::table.filter(user_groups::group_name.eq(group_name_b))
        .get_result(&**conn)?;
    let group_b_members =
        group_memberships::table.filter(group_memberships::group_id.eq(group_b.id))
            .select(group_memberships::user_id);

    let count = users::table.filter(users::id.eq(any(group_a_members)))
        .filter(users::id.eq(any(group_b_members)))
        .count()
        .get_result(&**conn)?;

    Ok(count)
}

pub fn join_user_group_by_name(conn: &Connection,
                               user: &User,
                               group_name: &str)
                               -> Result<GroupMembership> {
    use schema::{user_groups, group_memberships};
    use diesel::result::Error::DatabaseError;
    use diesel::result::DatabaseErrorKind::UniqueViolation;

    let group: UserGroup = user_groups::table.filter(user_groups::group_name.eq(group_name))
        .first(&**conn)?;

        // FIXME when diesel gets UPSERT, streamline this
    let membership: Option<GroupMembership> = diesel::insert(&GroupMembership {
            user_id: user.id,
            group_id: group.id,
            anonymous: false,
        }).into(group_memberships::table)
        .get_result(&**conn)
        .optional()
        .or_else(|e| match e {
            DatabaseError(UniqueViolation, _) => Ok(None),
            e => Err(e),
        })?;

    if let Some(membership) = membership {
        Ok(membership)
    } else {
        Ok(group_memberships::table
            .filter(group_memberships::user_id.eq(user.id)
                .and(group_memberships::group_id.eq(group.id)))
            .get_result(&**conn)?)
    }

}

pub fn remove_user_group_by_name(conn: &Connection,
                                 user_id: i32,
                                 group_name: &str)
                                 -> Result<bool> {
    use schema::{user_groups, group_memberships};

    let group: UserGroup = user_groups::table.filter(user_groups::group_name.eq(group_name))
        .first(&**conn)?;

    let count =
        diesel::delete(group_memberships::table.filter(group_memberships::user_id.eq(user_id)
                .and(group_memberships::group_id.eq(group.id)))).execute(&**conn)?;

    Ok(count == 1)
}

pub fn check_user_group(conn: &Connection, user_id: i32, group_name: &str) -> Result<bool> {
    use schema::{user_groups, group_memberships};

    if group_name == "" {
        return Ok(true);
    };

    let group: Option<UserGroup> = user_groups::table.filter(user_groups::group_name.eq(group_name))
        .get_result(&**conn)
        .optional()?;

    let group = if let Some(g) = group {
        g
    } else {
        return Err(ErrorKind::NoneResult.into());
    };

    let exists: Option<GroupMembership> =
        group_memberships::table.filter(group_memberships::user_id.eq(user_id))
            .filter(group_memberships::group_id.eq(group.id))
            .get_result(&**conn)
            .optional()?;

    Ok(exists.is_some())
}

pub fn get_users_by_group(conn: &Connection,
                          group_id: i32)
                          -> Result<Vec<(User, GroupMembership)>> {
    use schema::{group_memberships, users};

    let users: Vec<(User, GroupMembership)> = users::table.inner_join(group_memberships::table)
        .filter(group_memberships::group_id.eq(group_id))
        .get_results(&**conn)?;

    Ok(users)
}

pub fn get_group(conn: &Connection, group_name: &str) -> Result<Option<UserGroup>> {
    use schema::user_groups;

    let group: Option<(UserGroup)> =
        user_groups::table.filter(user_groups::group_name.eq(group_name))
            .get_result(&**conn)
            .optional()?;

    Ok(group)
}

pub fn all_groups(conn: &Connection) -> Result<Vec<UserGroup>> {
    use schema::user_groups;

    let groups = user_groups::table.order(user_groups::id)
        .get_results(&**conn)?;

    Ok(groups)
}

pub fn get_all(conn: &Connection)
               -> Result<(Vec<(User,
                               UserMetrics,
                               UserStats,
                               Vec<GroupMembership>,
                               i64,
                               Vec<String>)>,
                          Vec<UserGroup>,
                          Vec<PendingEmailConfirm>)> {
    use schema::{users, user_metrics, pending_email_confirms, group_memberships, user_stats,
                 sessions};

    let groups = all_groups(conn)?;

    let users: Vec<User> = users::table.get_results(&**conn)?;

    let mut overdues = vec![];

    for u in &users {
        overdues.push(quiz::count_overdue_items(conn, u.id)?);
    }

    let users_metrics: Vec<Vec<UserMetrics>> = user_metrics::table.get_results(&**conn)?
        .grouped_by(&users);

    let user_stats: Vec<Vec<UserStats>> = user_stats::table.get_results(&**conn)?
        .grouped_by(&users);

    let user_groups: Vec<Vec<GroupMembership>> = group_memberships::table.get_results(&**conn)?
        .grouped_by(&users);

    let sessions: Vec<Vec<Session>> = sessions::table.order(sessions::last_seen.desc())
        .get_results(&**conn)?
        .grouped_by(&users);

    let user_data: Vec<(User, UserMetrics, UserStats, Vec<GroupMembership>, i64, Vec<String>)> =
        users.into_iter()
            .zip(users_metrics.into_iter().zip(user_stats.into_iter()
                .zip(user_groups.into_iter().zip(overdues.into_iter().zip(sessions.into_iter())))))
            .map(|(u, (mut m, (mut s, (g, (o, sess)))))| {
                (u,
                 m.remove(0),
                 s.remove(0),
                 g,
                 o,
                 sess.into_iter().map(|s| s.last_seen.to_rfc3339()).collect())
            })
            .collect();

    let confirms: Vec<PendingEmailConfirm> = pending_email_confirms::table.get_results(&**conn)?;


    Ok((user_data, groups, confirms))
}

pub fn set_metrics(conn: &Connection,
                   metrics: &UpdateUserMetrics)
                   -> Result<Option<UserMetrics>> {
    use schema::user_metrics;

    let item =
        diesel::update(user_metrics::table.filter(user_metrics::id.eq(metrics.id))).set(metrics)
            .get_result(&**conn)
            .optional()?;

    Ok(item)
}

pub fn get_slackers(conn: &Connection, inactive: Duration) -> Result<Vec<(i32, String)>> {
    use schema::{users, sessions};
    use diesel::expression::all;

    let non_slackers = users::table.inner_join(sessions::table)
        .filter(sessions::last_seen.gt(chrono::UTC::now() - inactive))
        .select(users::id);

    let slackers: Vec<(i32, Option<String>)> = users::table.left_outer_join(sessions::table)
        .filter(users::email.is_not_null())
        .filter(users::id.ne(all(non_slackers)))
        .select((users::id, users::email))
        .distinct()
        .get_results(&**conn)?;

    let mut true_slackers = vec![];
    for (user_id, email) in slackers {

        let (next_existing_due, no_new_words, no_new_quizes) = quiz::things_left_to_do(conn,
                                                                                       user_id)?;

        if no_new_words && no_new_quizes && next_existing_due.is_none() {
            continue; // Nothing left to study , so he isn't a slacker
        }

        true_slackers.push((user_id, email.expect("We filtered NULL emails earlier")));

    }
    Ok(true_slackers)
}
