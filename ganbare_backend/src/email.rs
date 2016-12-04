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
use self::rust_email::{Mailbox};
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};

use schema::pending_email_confirms;
use super::*;

#[derive(RustcEncodable)]
struct EmailData<'a> { secret: &'a str, site_domain: &'a str }

pub fn send_confirmation<SOCK: ToSocketAddrs>(email : &str, secret : &str, mail_server: SOCK,
    email_origin_domain: &str, site_name: &str, hb_registry: &Handlebars) -> Result<EmailResponse> {
    
    impl<'a> ToJson for EmailData<'a> {
        fn to_json(&self) -> Json {
            let mut m: BTreeMap<String, Json> = BTreeMap::new();
            m.insert("secret".to_string(), self.secret.to_json());
            m.insert("site_domain".to_string(), self.site_domain.to_json());
            m.to_json()
        }
    }

    let data = EmailData { secret: secret, site_domain: site_name };
    let from_addr = format!("noreply@{}", email_origin_domain);
    let email = EmailBuilder::new()
        .to(email)
        .from(Mailbox {name: Some("gamba.re 応援団".into()), address: from_addr})
        .subject(&format!("[{}] Tervetuloa!", site_name))
        .html(hb_registry.render("email_confirm_email.html", &data)
            .chain_err(|| "Handlebars template render error!")?
            .as_ref())
        .build().expect("Building email shouldn't fail.");
    let mut mailer = SmtpTransportBuilder::new(mail_server)
        .chain_err(|| "Couldn't setup the email transport!")?
        .build();
    mailer.send(email)
        .chain_err(|| "Couldn't send email!")
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




