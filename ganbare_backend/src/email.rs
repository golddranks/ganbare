extern crate lettre;
extern crate handlebars;
extern crate email as rust_email;

use data_encoding;

use self::lettre::transport::smtp::response::Response as EmailResponse;
use self::lettre::transport::smtp::SmtpTransportBuilder;
use self::lettre::transport::EmailTransport;
use self::lettre::email::EmailBuilder;
use self::handlebars::Handlebars;
use std::net::ToSocketAddrs;
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use chrono::Duration;

use schema::pending_email_confirms;
use super::*;

#[derive(RustcEncodable)]
struct EmailData<'a> { secret: &'a str, site_link: &'a str, site_name: &'a str, }

impl<'a> ToJson for EmailData<'a> {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("secret".to_string(), self.secret.to_json());
        m.insert("site_link".to_string(), self.site_link.to_json());
        m.insert("site_name".to_string(), self.site_name.to_json());
        m.to_json()
    }
}

pub fn send_confirmation<SOCK: ToSocketAddrs>(email_addr : &str, secret : &str, mail_server: SOCK, username: &str, password: &str,
    site_name: &str, site_link: &str, hb_registry: &Handlebars, from: (&str, &str)) -> Result<EmailResponse> {

    let data = EmailData { secret, site_link, site_name };
    let email = EmailBuilder::new()
        .to(email_addr)
        .from(from)
        .subject(&format!("【{}】Tervetuloa!", site_name))
        .html(hb_registry.render("email_confirm_email.html", &data)
            .chain_err(|| "Handlebars template render error!")?
            .as_ref())
        .build().expect("Building email shouldn't fail.");
    let mut mailer = SmtpTransportBuilder::new(mail_server)
        .chain_err(|| "Couldn't setup the email transport!")?
        .encrypt()
        .credentials(username, password)
        .build();
    mailer.send(email)
        .chain_err(|| "Couldn't send email!")
}

pub fn send_pw_reset_email<SOCK: ToSocketAddrs>(secret: &ResetEmailSecrets, mail_server: SOCK, username: &str, password: &str,
    site_name: &str, site_link: &str, hb_registry: &Handlebars, from: (&str, &str)) -> Result<EmailResponse> {

    let data = EmailData { secret: &secret.secret, site_link, site_name };
    let email = EmailBuilder::new()
        .to(secret.email.as_str())
        .from(from)
        .subject(&format!("【{}】Salasanan vaihtaminen", site_name))
        .html(hb_registry.render("pw_reset_email.html", &data)
            .chain_err(|| "Handlebars template render error!")?
            .as_ref())
        .build().expect("Building email shouldn't fail.");
    let mut mailer = SmtpTransportBuilder::new(mail_server)
        .chain_err(|| "Couldn't setup the email transport!")?
        .encrypt()
        .credentials(username, password)
        .build();
    mailer.send(email)
        .chain_err(|| "Couldn't send email!")
}

pub fn send_freeform_email<'a, SOCK: ToSocketAddrs, ITER: Iterator<Item=&'a str>>(mail_server: SOCK, username: &str, password: &str,
    from: (&str, &str), to: ITER, subject: &str, body: &str) -> Result<()> {

    let mut mailer = SmtpTransportBuilder::new(mail_server)
        .chain_err(|| "Couldn't setup the email transport!")?
        .encrypt()
        .credentials(username, password)
        .build();

    for to in to {
    
        let email = EmailBuilder::new()
            .from(from)
            .subject(subject)
            .text(body)
            .to(to)
            .build().expect("Building email shouldn't fail.");

        let result = mailer.send(email)
            .chain_err(|| "Couldn't send!")?;
        info!("Sent freeform emails: {:?}!", result);
    }

    Ok(())
}



pub fn add_pending_email_confirm(conn : &PgConnection, email : &str, groups: &[i32]) -> Result<String> {
    let secret = data_encoding::base64url::encode(&session::fresh_token()?[..]);
    {
        let confirm = NewPendingEmailConfirm {
            email,
            secret: secret.as_ref(),
            groups
        };
        diesel::insert(&confirm)
            .into(pending_email_confirms::table)
            .execute(conn)
            .chain_err(|| "Error :(")?;
    }
    Ok(secret)
}

pub fn check_pending_email_confirm(conn : &PgConnection, secret : &str) -> Result<Option<(String, Vec<i32>)>> {
    let confirm : Option<PendingEmailConfirm> = pending_email_confirms::table
        .filter(pending_email_confirms::secret.eq(secret))
        .first(conn)
        .optional()?;

    Ok(confirm.map(|c| (c.email, c.groups)))
}

pub fn complete_pending_email_confirm(conn : &PgConnection, password : &str, secret : &str, pepper: &[u8]) -> Result<User> {

    let (email, group_ids) = try_or!(check_pending_email_confirm(&*conn, secret)?,
        else return Err(ErrorKind::NoSuchSess.into()));
    let user = user::add_user(&*conn, &email, password, pepper)?;

    for g in group_ids {
        user::join_user_group_by_id(&*conn, user.id, g)?
    }

    diesel::delete(pending_email_confirms::table
        .filter(pending_email_confirms::secret.eq(secret)))
        .execute(conn)
        .chain_err(|| "Couldn't delete the pending request.")?;

    Ok(user)
}

pub fn send_nag_emails<'a, SOCK: ToSocketAddrs>(conn: &PgConnection, how_old: chrono::Duration, mail_server: SOCK, username: &str, password: &str,
    site_name: &str, site_link: &str, /* hb_registry: &Handlebars, */ from: (&str, &str)) -> Result<()> {

    let slackers = user::get_slackers(conn, how_old)?;

    if slackers.len() == 0 { return Ok(()) }

    let mut mailer = SmtpTransportBuilder::new(mail_server)
        .chain_err(|| "Couldn't setup the email transport!")?
        .encrypt()
        .credentials(username, password)
        .build();

    for (user_id, email_addr) in slackers {
        println!("{:?} {:?}\n", user_id, email_addr);
        continue; // FIXME
        let data = EmailData { secret: "", site_link, site_name };
        let email = EmailBuilder::new()
            .to(email_addr.as_str())
            .from(from)
            .subject(&format!("【{}】Helou :3", site_name))
            .html("jeeah"/*hb_registry.render("slacker_heatenings.html", &data) // FIXME
                .chain_err(|| "Handlebars template render error!")?
                .as_ref()*/)
            .build().expect("Building email shouldn't fail.");

        let result = mailer.send(email)
            .chain_err(|| "Couldn't send!")?;
        info!("Sent slacker heatening emails: {:?}!", result);
    }

    Ok(())
}


