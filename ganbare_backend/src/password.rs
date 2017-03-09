extern crate dotenv;

use super::errors::*;
use super::models::Password;
use std::time::{Instant, Duration};

#[derive(Clone, Copy)]
pub struct HashedPassword {
    hash: [u8; 24],
    salt: [u8; 16],
    initial_rounds: i16,
    extra_rounds: i16,
}

impl From<Password> for HashedPassword {
    fn from(db_password: Password) -> Self {
        let mut hash = [0_u8; 24];
        let mut salt = [0_u8; 16];
        hash[..].clone_from_slice(&db_password.password_hash[..]);
        salt[..].clone_from_slice(&db_password.salt[..]);
        HashedPassword {
            hash: hash,
            salt: salt,
            initial_rounds: db_password.initial_rounds,
            extra_rounds: db_password.extra_rounds,
        }
    }
}


impl HashedPassword {
    pub fn into_db(self, user_id: i32) -> Password {
        Password {
            id: user_id,
            password_hash: (&self.hash[..]).into(),
            salt: (&self.salt[..]).into(),
            initial_rounds: self.initial_rounds,
            extra_rounds: self.extra_rounds,
        }
    }
}

fn pepper_salt_pw_hash(plaintext_pw: &str,
                       salt: [u8; 16],
                       initial_rounds: i16,
                       runtime_pepper: &[u8])
                       -> Result<HashedPassword> {
    use crypto::bcrypt::bcrypt;
    use crypto::sha2;
    use crypto::digest::Digest;

    let build_pepper = base64!("env:GANBARE_BUILDTIME_PEPPER");

    let mut hasher = sha2::Sha512::new();
    hasher.input_str(plaintext_pw);
    hasher.input(build_pepper);
    hasher.input(runtime_pepper);
    let mut peppered_pw = [0_u8; 64];
    hasher.result(&mut peppered_pw);
    let peppered_pw = peppered_pw;

    let mut output_hash = [0_u8; 24];
    bcrypt(initial_rounds as u32, &salt, &peppered_pw, &mut output_hash);
    Ok(HashedPassword {
           hash: output_hash,
           salt: salt,
           initial_rounds: initial_rounds,
           extra_rounds: 0,
       })
}

pub fn set_password(plaintext_pw: &str,
                    pepper: &[u8],
                    stretch_time: Duration)
                    -> Result<HashedPassword> {
    use rand::{OsRng, Rng};

    if plaintext_pw.len() < 8 {
        return Err(ErrorKind::PasswordTooShort.into());
    };
    if plaintext_pw.len() > 1024 {
        return Err(ErrorKind::PasswordTooLong.into());
    };

    let mut salt = [0_u8; 16];
    OsRng::new()?.fill_bytes(&mut salt);

    let mut rounds = 10;
    let start_time = Instant::now();
    let mut hashed_pw = pepper_salt_pw_hash(plaintext_pw, salt, rounds, pepper)?;
    let mut elapsed = Instant::now().duration_since(start_time);

    while elapsed < stretch_time {

        debug!("Not enough time elapsed ({:?}). Stretching more.", elapsed);
        rounds += 1;
        let start_time = Instant::now();
        hashed_pw = pepper_salt_pw_hash(plaintext_pw, salt, rounds, pepper)?;
        elapsed = Instant::now().duration_since(start_time);

    }

    Ok(hashed_pw)
}

pub fn stretch_password(strength_goal: i16, hashed_pw: HashedPassword) -> HashedPassword {
    use crypto::bcrypt::bcrypt;

    // We can regard the password hash as the output of the original creation function.
    let mut output_hash = hashed_pw.hash;
    let mut extra_rounds = hashed_pw.extra_rounds;

    while hashed_pw.initial_rounds + extra_rounds < strength_goal {
        let input = output_hash;
        bcrypt((hashed_pw.initial_rounds + extra_rounds) as u32,
               &hashed_pw.salt,
               &input,
               &mut output_hash);
        extra_rounds += 1;
    }
    HashedPassword {
        hash: output_hash,
        salt: hashed_pw.salt,
        initial_rounds: hashed_pw.initial_rounds,
        extra_rounds: extra_rounds,
    }
}

pub fn check_password(plaintext_pw: &str, pw_from_db: HashedPassword, pepper: &[u8]) -> Result<()> {
    use crypto::util::fixed_time_eq;
    let init_hash = pepper_salt_pw_hash(plaintext_pw,
                                        pw_from_db.salt,
                                        pw_from_db.initial_rounds,
                                        pepper)?;
    let strected_pw = stretch_password(pw_from_db.initial_rounds + pw_from_db.extra_rounds,
                                       init_hash);

    if fixed_time_eq(&strected_pw.hash, &pw_from_db.hash) {
        Ok(())
    } else {
        Err(ErrorKind::PasswordDoesntMatch.into())
    }
}


#[test]
fn test_set_check_password1() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);
    let pw = set_password("password", &pepper).unwrap();
    check_password("password", pw, &pepper).expect("Passwords should match!");
}

#[test]
fn test_set_check_password2() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);
    let pw = set_password("password1", &pepper).unwrap();
    if let Ok(()) = check_password("password2", pw, &pepper) {
        panic!("Passwords shouldn't match!");
    }
}

#[test]
fn test_set_stretch_password1() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);

    let init_pw = set_password("daggerfish", &pepper).unwrap();
    println!("hashed init_hash.");
    let stretched_pw_0 = stretch_password(11, init_pw);
    println!("stretched 10 → 11.");
    let stretched_pw_1 = stretch_password(12, stretched_pw_0);
    println!("stretched 11 → 12.");

    let stretched_pw_2 = stretch_password(12, stretched_pw_1);
    println!("stretched 10 → 12.");

    assert_eq!(stretched_pw_1.hash, stretched_pw_2.hash);
    assert_eq!(stretched_pw_1.extra_rounds, stretched_pw_2.extra_rounds);
}



#[test]
fn test_set_stretch_password2() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);

    let init_pw_1 = set_password("swordfish", &pepper).unwrap();
    println!("hashed init_hash.");
    let init_pw_2 = stretch_password(10, init_pw_1);
    println!("stretched 10 → 10.");

    assert_eq!(init_pw_1.hash, init_pw_2.hash);
    assert_eq!(0, init_pw_2.extra_rounds);
}


#[test]
fn test_set_stretch_password3() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);

    let init_pw = set_password("schwertfisch", &pepper).unwrap();
    println!("hashed init_hash.");
    let stretched_pw_0 = stretch_password(11, init_pw);
    println!("stretched 10 → 11.");
    let stretched_pw_1 = stretch_password(12, stretched_pw_0);
    println!("stretched 11 → 11.");

    let stretched_pw_2 = stretch_password(12, init_pw);
    println!("stretched 10 → 11.");

    assert_eq!(stretched_pw_1.hash, stretched_pw_2.hash);
    assert_eq!(stretched_pw_1.extra_rounds, stretched_pw_2.extra_rounds);
}

#[test]
fn test_set_stretch_check_password1() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);

    let init_pw = set_password("miekkakala", &pepper).unwrap();
    println!("hashed init_hash.");
    let stretched_pw = stretch_password(11, init_pw);
    println!("stretched 10 → 11.");

    check_password("miekkakala", stretched_pw, &pepper).expect("Passwords should match!");
}

#[test]
fn test_set_stretch_check_password2() {
    use rand::{StdRng, Rng};
    let mut pepper = [0_u8; 32];
    StdRng::new().unwrap().fill_bytes(&mut pepper);

    let init_pw = set_password("miekkakala", &pepper).unwrap();
    println!("hashed init_hash.");
    let stretched_pw = stretch_password(11, init_pw);
    println!("stretched 10 → 11.");

    if let Ok(()) = check_password("tikarikala", stretched_pw, &pepper) {
        panic!("Passwords shouldn't match!");
    }
}
