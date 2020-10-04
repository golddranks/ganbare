extern crate lettre;
extern crate sharp_pencil as pencil;

use std::time::Duration;

use self::lettre::email::{EmailBuilder, Email};
use self::pencil::Handlebars;
use std::sync::RwLock;
use std::collections::VecDeque;
use email::lettre::email::SendableEmail;

use schema::pending_email_confirms;
use super::*;

#[derive(Serialize)]
struct EmailData<'a> {
    secret: &'a str,
    hmac: &'a str,
    site_link: &'a str,
    site_name: &'a str,
}
/*
impl<'a> ToJson for EmailData<'a> {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("secret".to_string(), self.secret.to_json());
        m.insert("hmac".to_string(), self.hmac.to_json());
        m.insert("site_link".to_string(), self.site_link.to_json());
        m.insert("site_name".to_string(), self.site_name.to_json());
        m.to_json()
    }
}*/

fn enqueue_mail(email: Email, queue: &RwLock<VecDeque<Email>>) -> Result<()> {

    info!("Enqueuing mail to {:?}", email.to_addresses());

    match queue.write() {
        Ok(mut q) => {
            q.push_back(email);
            Ok(())
        }
        _ => Err(Error::from_kind("Couldn't open the email queue for writing.".into())),
    }
}

pub fn send_confirmation(queue: &RwLock<VecDeque<Email>>,
                         email_addr: &str,
                         secret: &str,
                         hmac: &str,
                         site_name: &str,
                         site_link: &str,
                         hb_registry: &Handlebars,
                         from: (&str, &str))
                         -> Result<()> {

    let data = EmailData {
        secret: secret,
        hmac: hmac,
        site_link: site_link,
        site_name: site_name,
    };
    let email = EmailBuilder::new()
        .to(email_addr)
        .from(from)
        .subject(&format!("【{}】Tervetuloa!", site_name))
        .html(hb_registry.render("email_confirm_email.html", &data)
                  .chain_err(|| "Handlebars template render error!")?
                  .as_ref())
        .build()
        .expect("Building email shouldn't fail.");
    enqueue_mail(email, queue)?;
    Ok(())
}

pub fn send_pw_reset_email(queue: &RwLock<VecDeque<Email>>,
                           secret: &ResetEmailSecrets,
                           hmac: &str,
                           site_name: &str,
                           site_link: &str,
                           hb_registry: &Handlebars,
                           from: (&str, &str))
                           -> Result<()> {

    let data = EmailData {
        secret: &secret.secret,
        hmac: hmac,
        site_link: site_link,
        site_name: site_name,
    };
    let email = EmailBuilder::new()
        .to(secret.email.as_str())
        .from(from)
        .subject(&format!("【{}】Salasanan vaihtaminen", site_name))
        .html(hb_registry.render("pw_reset_email.html", &data)
                  .chain_err(|| "Handlebars template render error!")?
                  .as_ref())
        .build()
        .expect("Building email shouldn't fail.");
    enqueue_mail(email, queue)?;
    Ok(())
}

pub fn send_freeform_email<'a, ITER: Iterator<Item = &'a str>>(queue: &RwLock<VecDeque<Email>>,
                                                               from: (&str, &str),
                                                               to: ITER,
                                                               subject: &str,
                                                               body: &str)
                                                               -> Result<()> {

    for to in to {

        let email = EmailBuilder::new()
            .from(from)
            .subject(subject)
            .text(body)
            .to(to)
            .build()
            .expect("Building email shouldn't fail.");

        enqueue_mail(email, queue)?;
    }

    Ok(())
}



pub fn add_pending_email_confirm(conn: &Connection,
                                 hmac_key: &[u8],
                                 email: &str,
                                 groups: &[i32])
                                 -> Result<(String, String)> {
    let (secret, hmac) = session::new_token_and_hmac(hmac_key)?;

    {
        let confirm = NewPendingEmailConfirm {
            email: email,
            secret: secret.as_ref(),
            groups: groups,
        };
        diesel::insert_into(pending_email_confirms::table).values(&confirm)
            .execute(&**conn)
            .chain_err(|| "Error :(")?;
    }
    Ok((secret, hmac))
}

pub fn get_all_pending_email_confirms(conn: &Connection) -> Result<Vec<String>> {
    let emails: Vec<String> = pending_email_confirms::table.select(pending_email_confirms::email)
        .get_results(&**conn)?;

    Ok(emails)
}

pub fn check_pending_email_confirm(conn: &Connection,
                                   secret: &str)
                                   -> Result<Option<(String, Vec<i32>)>> {
    let confirm: Option<PendingEmailConfirm> =
        pending_email_confirms::table.filter(pending_email_confirms::secret.eq(secret))
            .first(&**conn)
            .optional()?;

    Ok(confirm.map(|c| (c.email, c.groups)))
}

pub fn complete_pending_email_confirm(conn: &Connection,
                                      password: &str,
                                      secret: &str,
                                      pepper: &[u8],
                                      stretching_time: Duration)
                                      -> Result<User> {

    let (email, group_ids) = try_or!(check_pending_email_confirm(&conn, secret)?,
        else return Err(ErrorKind::NoSuchSess.into()));
    let user = user::add_user(&*conn, &email, password, pepper, stretching_time)?;

    for g in group_ids {
        user::join_user_group_by_id(conn, user.id, g)?;
    }

    diesel::delete(pending_email_confirms::table
        .filter(pending_email_confirms::secret.eq(secret)))
        .execute(&**conn)
        .chain_err(|| "Couldn't delete the pending request.")?;

    Ok(user)
}

pub fn clean_old_pendings(conn: &Connection, duration: chrono::Duration) -> Result<usize> {
    let deadline = chrono::offset::Utc::now() - duration;
    diesel::delete(pending_email_confirms::table.filter(pending_email_confirms::added.lt(deadline)))
        .execute(&**conn)
        .chain_err(|| "Couldn't delete the old pending requests.")
}

pub fn send_nag_emails(queue: &RwLock<VecDeque<Email>>,
                       conn: &Connection,
                       how_old: chrono::Duration,
                       nag_grace_period: chrono::Duration,
                       site_name: &str,
                       site_link: &str,
                       hb_registry: &Handlebars,
                       from: (&str, &str))
                       -> Result<()> {

    let slackers = user::get_slackers(conn, how_old)?;

    if slackers.is_empty() {
        return Ok(());
    }

    for (user_id, email_addr) in slackers {

        use schema::user_stats;

        let mut stats: UserStats = user_stats::table.filter(user_stats::id.eq(user_id))
            .get_result(&**conn)?;

        if !user::check_user_group(conn, user_id, "nag_emails")? {
            continue; // We don't send emails to users that don't belong to the "nag_emails" group.
        }

        let last_nag = stats.last_nag_email.unwrap_or_else(|| chrono::MIN_DATE.and_hms(0, 0, 0));

        if last_nag > chrono::offset::Utc::now() - nag_grace_period {
            continue; // We have sent a nag email recently
        }

        let data = EmailData {
            secret: "",
            hmac: "",
            site_link: site_link,
            site_name: site_name,
        };
        let email = EmailBuilder::new()
            .to(email_addr.as_str())
            .from(from)
            .subject(&format!("【{}】Minne katosit? (´・ω・`)", site_name))
            .html(hb_registry.render("slacker_heatenings.html", &data) // FIXME
                .chain_err(|| "Handlebars template render error!")?
                .as_ref())
            .build()
            .expect("Building email shouldn't fail.");

        enqueue_mail(email, queue)?;

        stats.last_nag_email = Some(chrono::offset::Utc::now());
        let _: UserStats = stats.save_changes(&**conn)?;

        info!("Sent slacker heatening email to {}!", email_addr);
    }

    Ok(())
}
