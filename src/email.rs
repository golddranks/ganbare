extern crate lettre;
extern crate handlebars;
extern crate email;

use super::errors::{Result, ChainErr};
use self::lettre::transport::smtp::response::Response as EmailResponse;
use self::lettre::transport::smtp::SmtpTransportBuilder;
use self::lettre::transport::EmailTransport;
use self::lettre::email::EmailBuilder;
use self::handlebars::Handlebars;
use std::net::ToSocketAddrs;
use self::email::{Mailbox};
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};

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
