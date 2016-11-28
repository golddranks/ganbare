
use super::*;
use std::net::IpAddr;
use std::mem;

pub const SESSID_BITS : usize = 128;

pub fn fresh_token() -> Result<[u8; SESSID_BITS/8]> {
    use rand::{Rng, OsRng};
    let mut session_id = [0_u8; SESSID_BITS/8];
    OsRng::new().chain_err(|| "Unable to connect to the system random number generator!")?.fill_bytes(&mut session_id);
    Ok(session_id)
}

pub fn to_hex(sess : &Session) -> String {
    use data_encoding::base16;
    match sess.proposed_token {
        Some(ref t) => base16::encode(t),
        None => base16::encode(&sess.sess_token),
    }
    
}

fn token_to_bin(sessid : &str) -> Result<Vec<u8>> {
    use data_encoding::base16;
    if sessid.len() == SESSID_BITS/4 {
        base16::decode(sessid.as_bytes()).chain_err(|| ErrorKind::BadSessId)
    } else {
        Err(ErrorKind::BadSessId.to_err())
    }
}

fn update_token(conn: &PgConnection, mut sess: Session) -> Result<Session> {
    sess.sess_token = sess.proposed_token.ok_or(ErrorKind::DatabaseOdd("Bug: This function shouldn't have been called if proposed token didn't exist.").to_err())?;
    sess.proposed_token = None;
    let new_sess = sess.save_changes(conn)?;
    Ok(new_sess)
}


pub fn check(conn : &PgConnection, token_hex : &str) -> Result<Option<(User, Session)>> {
    use schema::{users, sessions};
    use diesel::ExpressionMethods;

    let token = token_to_bin(token_hex)?;

    let user_sess: Option<(User, Session)> = users::table
        .inner_join(sessions::table)
        .filter(sessions::sess_token.eq(&token).or(sessions::proposed_token.eq(&token)))
        .get_result(conn)
        .optional()?;

    if let Some((user, mut sess)) = user_sess {

        if sess.proposed_token == Some(token) {
            sess = update_token(conn, sess)?;
        }
            
        Ok(Some((user, sess)))
    } else {
        Ok(None)
    }
} 

pub fn refresh(conn : &PgConnection, sess : &mut Session, ip : IpAddr) -> Result<()> {
    use diesel::SaveChangesDsl;

    sess.last_ip.truncate(0);

    match ip {
        IpAddr::V4(ip) => { sess.last_ip.extend(&ip.octets()[..]) },
        IpAddr::V6(ip) => { sess.last_ip.extend(&ip.octets()[..]) },
    };

    let proposed_token = fresh_token()?;

    let mut vec = if let Some(mut vec) = sess.proposed_token.take() { vec.truncate(0); vec } else { vec![] };
    vec.extend(proposed_token.as_ref());

    sess.proposed_token = Some(vec);
    sess.last_seen = chrono::UTC::now();

    let updated_sess = sess.save_changes(conn)?;

    mem::replace(sess, updated_sess);

    Ok(())
} 

pub fn end(conn : &PgConnection, token_hex : &str) -> Result<Option<()>> {
    use schema::sessions;

    let token = token_to_bin(token_hex)?;

    let deleted = diesel::delete(sessions::table
            .filter(sessions::sess_token.eq(&token).or(sessions::proposed_token.eq(&token)))
        )
        .execute(conn)
        .optional()?;
    Ok(deleted.map(|_| ()))
} 

pub fn start(conn : &PgConnection, user : &User, ip : IpAddr) -> Result<Session> {
    use schema::sessions;

    let new_sessid = fresh_token()?;

    let ip_as_bytes = match ip {
        IpAddr::V4(ip) => { ip.octets()[..].to_vec() },
        IpAddr::V6(ip) => { ip.octets()[..].to_vec() },
    };

    let new_sess = NewSession {
        sess_token: &new_sessid,
        proposed_token: None,
        user_id: user.id,
        started: chrono::UTC::now(),
        last_seen: chrono::UTC::now(),
        last_ip: ip_as_bytes,
    };

    diesel::insert(&new_sess)
        .into(sessions::table)
        .get_result(conn)
        .chain_err(|| "Couldn't start a session!") // TODO if the session id already exists, this is going to fail? (A few-in-a 2^128 change, though...)
}
