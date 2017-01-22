
use super::*;
use std::net::IpAddr;
use std::thread;
use std::time::Duration;
use rand::{Rng, OsRng};

pub const SESSID_BITS: usize = 128;

pub fn fresh_token() -> Result<[u8; SESSID_BITS / 8]> {
    use rand::{Rng, OsRng};
    let mut session_id = [0_u8; SESSID_BITS / 8];
    OsRng::new()
        .chain_err(|| "Unable to connect to the system random number generator!")?
        .fill_bytes(&mut session_id);
    Ok(session_id)
}

pub fn to_hex(sess: &Session) -> String {
    use data_encoding::base16;
    base16::encode(&sess.proposed_token)
}

pub fn bin_to_hex(bin: &[u8]) -> String {
    // FIXME remove this debug-only function
    use data_encoding::base16;
    base16::encode(bin)
}

fn token_to_bin(sessid: &str) -> Result<Vec<u8>> {
    use data_encoding::base16;
    if sessid.len() == SESSID_BITS / 4 {
        base16::decode(sessid.as_bytes()).chain_err(|| ErrorKind::BadSessId)
    } else {
        Err(ErrorKind::BadSessId.to_err())
    }
}

pub fn clean_old_sessions(conn: &PgConnection, how_old: chrono::Duration) -> Result<usize> {
    use schema::sessions;

    let deleted_count = diesel::delete(sessions::table
        .filter(sessions::last_seen.lt(chrono::UTC::now()-how_old)))
        .execute(conn)?;

    Ok(deleted_count)
}

pub fn check(conn: &PgConnection, token_hex: &str, ip: IpAddr) -> Result<Option<(User, Session)>> {
    use schema::{users, sessions};
    use diesel::ExpressionMethods;

    let token = match token_to_bin(token_hex) {
        Ok(t) => t,
        Err(_) => return Ok(None), // If the token isn't in a valid format, just return "None".
    };

    let result;
    loop {
        // CAS loop. Try to update the DB until it succeeds.

        let user_sess: Option<(User, Session)> = users::table.inner_join(sessions::table)
            .filter(sessions::current_token.eq(&token)
                .or(sessions::proposed_token.eq(&token)
                    .or(sessions::retired_token.eq(&token))))
            .get_result(conn)
            .optional()?;

        if let Some((user, mut sess)) = user_sess {

            let expect_version = sess.access_version;

            if sess.proposed_token == token {
                // User seems to adopted the new, proposed token! Upgrading it to the current token.

                sess.access_version += 1; // Only updating tokens will increment the access version.
                // Note that this allows concurrent updates to last_ip and last_seen.
                sess.retired_token.truncate(0);
                sess.retired_token.extend(&sess.current_token);
                sess.current_token.truncate(0);
                sess.current_token.extend(&sess.proposed_token);
                sess.proposed_token.truncate(0);
                sess.proposed_token.extend(&fresh_token()?);
            }

            sess.last_ip.truncate(0);
            match ip {
                IpAddr::V4(ip) => sess.last_ip.extend(&ip.octets()[..]),
                IpAddr::V6(ip) => sess.last_ip.extend(&ip.octets()[..]),
            };
            sess.last_seen = chrono::UTC::now();

            let rows_updated = diesel::update(sessions::table.filter(sessions::id.eq(sess.id))
                    .filter(sessions::access_version.eq(expect_version))).set(&sess)
                .execute(conn)?;

            if rows_updated == 0 {
                continue; // Failed to commit; some other connection commited new tokens
            } else {
                result = Ok(Some((user, sess))); // Successfully commited
                break;
            }

        } else {
            warn!("Somebody tried to open a page with wrong credentials! (Either a bug or a \
                   hacking attempt.)");
            // Punishment sleep for wrong credentials
            thread::sleep(Duration::from_millis(20 +
                                                OsRng::new()
                .expect("If we can't get OS RNG, we might as well crash.")
                .gen_range(0, 5)));
            result = Ok(None);
            break;
        }
    }

    result
}

pub fn end(conn: &PgConnection, sess: &Session) -> Result<Option<()>> {
    use schema::sessions;

    let deleted_count =
        diesel::delete(sessions::table.filter(sessions::id.eq(sess.id))).execute(conn)?;
    Ok(if deleted_count != 1 {
        warn!("Somebody tried to log out with wrong credentials! (Either a bug or a hacking \
               attempt.)");
        // Punishment sleep for wrong credentials
        thread::sleep(Duration::from_millis(20 +
                                            OsRng::new()
            .expect("If we can't get OS RNG, we might as well crash.")
            .gen_range(0, 5)));
        None
    } else {
        Some(())
    })
}

pub fn start(conn: &PgConnection, user: &User, ip: IpAddr) -> Result<Session> {
    use schema::sessions;

    let new_proposed_token = fresh_token()?;
    let pseudo_current_token = fresh_token()?;
    let pseudo_retired_token = fresh_token()?;

    let ip_as_bytes = match ip {
        IpAddr::V4(ip) => ip.octets()[..].to_vec(),
        IpAddr::V6(ip) => ip.octets()[..].to_vec(),
    };

    let new_sess = NewSession {
        proposed_token: &new_proposed_token,
        current_token: &pseudo_retired_token,
        retired_token: &pseudo_current_token,
        user_id: user.id,
        started: chrono::UTC::now(),
        last_seen: chrono::UTC::now(),
        last_ip: &ip_as_bytes,
    };

    // if the session id already exists,
    // this is going to fail? (A few-in-a 2^128 change, though...)
    diesel::insert(&new_sess)
        .into(sessions::table)
        .get_result(conn)
        .chain_err(|| "Couldn't start a session!")
}
