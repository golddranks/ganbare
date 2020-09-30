use super::*;
use std::thread;
use std::time::Duration;
use rand::{Rng, OsRng};
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha2::Sha512;
use chrono::{self, DateTime, offset::Utc};
use data_encoding::base64url::{encode_nopad, decode_nopad};

pub const SESSID_BITS: usize = 128;
pub const HMAC_BITS: usize = 512;

#[derive(Debug, Clone)]
pub struct UserSession {
    pub sess_id: i32,
    pub user_id: i32,
    pub refreshed: DateTime<Utc>,
    pub refresh_now: bool,
    pub token: Vec<u8>,
    pub refresh_count: i32,
}

pub fn new_token_and_hmac(hmac_key: &[u8]) -> Result<(String, String)> {
    use crypto::hmac::Hmac;
    use crypto::mac::Mac;
    use crypto::sha2::Sha512;

    let token = session::fresh_token()?;
    let mut hmac_checker = Hmac::new(Sha512::new(), hmac_key);
    hmac_checker.input(&token[..]);
    let hmac = hmac_checker.result();
    let token_base64url = encode_nopad(&token[..]);
    let hmac_base64url = encode_nopad(hmac.code());

    Ok((token_base64url, hmac_base64url))
}

pub fn verify_token(token_base64url: &str, hmac_base64url: &str, hmac_key: &[u8]) -> Result<bool> {
    use crypto::hmac::Hmac;
    use crypto::mac::{Mac, MacResult};
    use crypto::sha2::Sha512;

    let token = decode_nopad(token_base64url.as_bytes())?;
    let hmac = decode_nopad(hmac_base64url.as_bytes())?;

    let mut hmac_checker = Hmac::new(Sha512::new(), hmac_key);
    hmac_checker.input(token.as_slice());
    if hmac_checker.result() == MacResult::new(hmac.as_slice()) {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn fresh_token() -> Result<[u8; SESSID_BITS / 8]> {
    use rand::{Rng, OsRng};
    let mut session_id = [0_u8; SESSID_BITS / 8];
    OsRng::new()
        .chain_err(|| "Unable to connect to the system random number generator!")?
        .fill_bytes(&mut session_id);
    Ok(session_id)
}

pub fn get_hmac_for_sess(session_id: &str,
                         user_id: &str,
                         refreshed: &str,
                         refresh_count: &str,
                         token: &[u8],
                         secret_key: &[u8])
                         -> String {
    let mut hmac_maker = Hmac::new(Sha512::new(), secret_key);
    hmac_maker.input(session_id.as_bytes());
    hmac_maker.input(user_id.as_bytes());
    hmac_maker.input(refreshed.as_bytes());
    hmac_maker.input(refresh_count.as_bytes());
    hmac_maker.input(token);
    encode_nopad(hmac_maker.result().code())
}

pub fn verify_hmac_for_sess_secret(secret: &[u8], refresh_count: i32, token: &[u8]) -> bool {
    use byteorder::WriteBytesExt;

    let mut refresh_times_bytes = [0_u8; 4];
    (&mut refresh_times_bytes[..]).write_i32::<byteorder::LittleEndian>(refresh_count)
        .expect("We should be able to write to the memory we just allocated!");

    let mut hmac_maker = Hmac::new(Sha512::new(), secret);
    hmac_maker.input(&refresh_times_bytes[..]);

    hmac_maker.result() == MacResult::new(token)
}

pub fn get_hmac_for_sess_secret(secret: &[u8], refresh_count: i32) -> Vec<u8> {
    use byteorder::WriteBytesExt;

    let mut refresh_times_bytes = [0_u8; 4];
    (&mut refresh_times_bytes[..]).write_i32::<byteorder::LittleEndian>(refresh_count)
        .expect("We should be able to write to the memory we just allocated!");

    let mut hmac_maker = Hmac::new(Sha512::new(), secret);
    hmac_maker.input(&refresh_times_bytes[..]);
    hmac_maker.result().code().to_owned()
}

pub fn clean_old_sessions(conn: &Connection, how_old: chrono::Duration) -> Result<usize> {
    use schema::sessions;

    let deleted_count =
        diesel::delete(sessions::table.filter(sessions::last_seen.lt(chrono::offset::Utc::now() -
                                                                     how_old))).execute(&**conn)?;

    Ok(deleted_count)
}

pub fn check_integrity(sess_id_str: &str,
                       user_id_str: &str,
                       refreshed_str: &str,
                       hmac: &str,
                       token_base64url: &str,
                       refresh_count_str: &str,
                       secret_key: &[u8])
                       -> Result<UserSession> {
    let sess_id = sess_id_str.parse()?;
    let user_id = user_id_str.parse()?;
    let refresh_count = refresh_count_str.parse()?;
    let refreshed = DateTime::parse_from_rfc3339(refreshed_str)?.with_timezone(&Utc);

    let hmac = decode_nopad(hmac.as_bytes())?;
    let token = decode_nopad(token_base64url.as_bytes())?;

    let mut hmac_checker = Hmac::new(Sha512::new(), secret_key);
    hmac_checker.input(sess_id_str.as_bytes());
    hmac_checker.input(user_id_str.as_bytes());
    hmac_checker.input(refreshed_str.as_bytes());
    hmac_checker.input(refresh_count_str.as_bytes());
    hmac_checker.input(token.as_slice());

    if hmac_checker.result() == MacResult::new_from_owned(hmac) {
        Ok(UserSession {
               sess_id: sess_id,
               user_id: user_id,
               refreshed: refreshed,
               refresh_now: false,
               token: token,
               refresh_count: refresh_count,
           })
    } else {
        warn!("The HMAC doesn't agree with the cookie!");
        bail!(ErrorKind::AuthError)
    }
}

use helpers::Cache;

pub fn check(sess: &UserSession, logout_cache: &Cache<i32, UserSession>) -> Result<bool> {
    if sess.refreshed > chrono::offset::Utc::now() - chrono::Duration::minutes(5) {
        if logout_cache.get(&sess.sess_id)?.is_some() {
            Ok(false) // User was recently logged out so don't trust their cookie!
        } else {
            Ok(true)
        }
    } else {
        Ok(false) // The cookie is over 5 minutes old so don't trust it.
    }
}

fn update_user_last_seen(conn: &Connection, user_id: i32, last_seen: chrono::DateTime<Utc>) -> Result<()> {
    use schema::users;

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::last_seen.eq(last_seen))
        .execute(&**conn)?;
    Ok(())
}

pub fn db_check(conn: &Connection,
                sess: &UserSession,
                sess_expire: chrono::Duration)
                -> Result<Option<UserSession>> {
    use schema::{sessions};

    time_it!{"session::db_check", {

        let oldest_viable = chrono::offset::Utc::now() - sess_expire;
        if sess.refreshed < oldest_viable {
            return Ok(None); // The session is expired
        }
        
        let session_refreshed = chrono::offset::Utc::now();

        let db_sess: Option<Session> = diesel::update(sessions::table
            .filter(
                sessions::id.eq(sess.sess_id)
                    .and(sessions::user_id.eq(sess.user_id))
                    .and(sessions::refresh_count.eq(sess.refresh_count))
                )
            )
            .set((sessions::last_seen.eq(session_refreshed),
                sessions::refresh_count.eq(sessions::refresh_count+1)))
            .get_result(&**conn)
            .optional()?;

        // If updating session succeed?
        match db_sess {
            // If it did, verify the session against the secret in DB
            Some(db_sess) => {
                if verify_hmac_for_sess_secret(db_sess.secret.as_slice(), db_sess.refresh_count-1, &sess.token) {

                    update_user_last_seen(conn, db_sess.user_id, session_refreshed)?;

                    Ok(Some(UserSession {
                        refreshed: session_refreshed,
                        user_id: db_sess.user_id,
                        sess_id: db_sess.id,
                        refresh_now: true,
                        token: get_hmac_for_sess_secret(db_sess.secret.as_slice(), db_sess.refresh_count),
                        refresh_count: db_sess.refresh_count,
                    }))
                } else {
                    Ok(None) // The token didn't match
                }

            }
            // If it didn't there might have been an concurrent update. Let's try again with incremented refresh count
            None => {
                let db_fresher_sess: Option<Session> = sessions::table
                    .filter(
                        sessions::id.eq(sess.sess_id)
                            .and(sessions::user_id.eq(sess.user_id))
                            .and(sessions::refresh_count.eq(sess.refresh_count+1))
                        )
                    .get_result(&**conn)
                    .optional()?;
                if let Some(db_sess) = db_fresher_sess { // The DB session was updated concurrently and this request had stale info
                    if verify_hmac_for_sess_secret(db_sess.secret.as_slice(), db_sess.refresh_count-1, &sess.token) {

                        update_user_last_seen(conn, db_sess.user_id, session_refreshed)?;
    
                        Ok(Some(UserSession {
                            refreshed: db_sess.last_seen,
                            user_id: db_sess.user_id,
                            sess_id: db_sess.id,
                            refresh_now: true,
                            token: get_hmac_for_sess_secret(db_sess.secret.as_slice(), db_sess.refresh_count),
                            refresh_count: db_sess.refresh_count,
                        }))
                    } else {
                        Ok(None) // The token didn't match
                    }
                } else {
                    Ok(None) // Session doesn't simply exist. The user has logged out, deleted or something.
                }
            }
        }
    }} // time_it ends
}

pub fn end(conn: &Connection, sess_id: i32) -> Result<Option<()>> {
    use schema::sessions;

    let deleted_count =
        diesel::delete(sessions::table.filter(sessions::id.eq(sess_id))).execute(&**conn)?;
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

pub fn start(conn: &Connection, user: &User) -> Result<UserSession> {
    use schema::sessions;

    let sess_secret = fresh_token()?;

    let session_started = chrono::offset::Utc::now();

    let new_sess = NewSession {
        user_id: user.id,
        started: session_started,
        last_seen: session_started,
        secret: &sess_secret[..],
    };

    update_user_last_seen(conn, user.id, session_started)?;

    let db_sess: Session = diesel::insert(&new_sess).into(sessions::table)
        .get_result(&**conn)
        .chain_err(|| "Couldn't start a session!")?;

    Ok(UserSession {
           user_id: user.id,
           sess_id: db_sess.id,
           refreshed: db_sess.last_seen,
           refresh_now: true,
           token: get_hmac_for_sess_secret(&sess_secret[..], 0),
           refresh_count: 0,
       })
}
