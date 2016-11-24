#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]

pub extern crate ganbare;
pub extern crate pencil;

extern crate dotenv;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
#[macro_use]  extern crate lazy_static;
#[macro_use]  extern crate mime;
extern crate time;
extern crate rustc_serialize;
extern crate rand;
extern crate chrono;
extern crate unicode_normalization;
extern crate regex;

use pencil::{Pencil};


#[macro_use]
mod helpers {

use std;
use std::env;
use dotenv;
use std::net::{SocketAddr, ToSocketAddrs};
use ganbare;
use ganbare::PgConnection;
use hyper::header::{SetCookie, CookiePair, Cookie};
use std::collections::BTreeMap;
use pencil::{Request, Response, abort};
use ganbare::models::{User, Session};
use ganbare::errors::{ErrorKind};
use std::net::IpAddr;
use time;
use pencil::PencilError;
use pencil;
use std::result::Result as StdResult;
use ganbare::errors::Result as Result;
use rustc_serialize::base64::FromBase64;

lazy_static! {
 
    pub static ref DATABASE_URL : String = { dotenv::dotenv().ok(); env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    pub static ref SITE_DOMAIN : String = { dotenv::dotenv().ok(); env::var("GANBARE_SITE_DOMAIN")
        .expect("GANBARE_SITE_DOMAIN: Set the site domain! (Without it, the cookies don't work.)") };

    pub static ref EMAIL_SERVER : SocketAddr = { dotenv::dotenv().ok();
        let binding = env::var("GANBARE_EMAIL_SERVER")
        .expect("GANBARE_EMAIL_SERVER: Specify an outbound email server, like this: mail.yourisp.com:25");
        binding.to_socket_addrs().expect("Format: domain:port").next().expect("Format: domain:port") };

    pub static ref EMAIL_DOMAIN : String = { dotenv::dotenv().ok(); env::var("GANBARE_EMAIL_DOMAIN")
        .unwrap_or_else(|_|  env::var("GANBARE_SITE_DOMAIN").unwrap_or_else(|_| "".into())) };

    pub static ref SERVER_BINDING : SocketAddr = { dotenv::dotenv().ok();
        let binding = env::var("GANBARE_SERVER_BINDING")
        .unwrap_or_else(|_| "localhost:8080".into());
        binding.to_socket_addrs().expect("GANBARE_SERVER_BINDING: Format: domain:port").next()
        .expect("GANBARE_SERVER_BINDING: Format: domain:port") };

    pub static ref JQUERY_URL : String = { dotenv::dotenv().ok(); env::var("GANBARE_JQUERY")
        .unwrap_or_else(|_| "/static/js/jquery.min.js".into()) };

    pub static ref AUDIO_DIR : String = { dotenv::dotenv().ok(); env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "audio".into()) };

    pub static ref IMAGES_DIR : String = { dotenv::dotenv().ok(); env::var("GANBARE_IMAGES_DIR")
        .unwrap_or_else(|_| "images".into()) };

    pub static ref RUNTIME_PEPPER : Vec<u8> = { dotenv::dotenv().ok();
        let pepper = env::var("GANBARE_RUNTIME_PEPPER")
        .expect("Environmental variable GANBARE_RUNTIME_PEPPER must be set! (format: 256-bit random value encoded as base64)")
        .from_base64().expect("Environmental variable GANBARE_RUNTIME_PEPPER isn't valid Base64!");
        if pepper.len() != 32 { panic!("The value must be 256-bit, that is, 32 bytes long!") }; pepper
    };

}

pub fn db_connect() -> Result<PgConnection> {
    ganbare::db_connect(&*DATABASE_URL)
}


pub fn get_cookie(cookies : &Cookie) -> Option<&str> {
    for c in cookies.0.iter() {
        if c.name == "session_id" {
            return Some(c.value.as_ref());
        }
    };
    None
}

pub fn new_template_context() -> BTreeMap<String, String> {
    let mut ctx = BTreeMap::new();
    ctx.insert("title".to_string(), "akusento.ganba.re".to_string());
    ctx.insert("jquery_url".to_string(), JQUERY_URL.to_string());
    ctx
}

pub fn get_user(conn : &PgConnection, req : &Request) -> Result<Option<(User, Session)>> {
    if let Some(session_id) = req.cookies().and_then(get_cookie) {
        ganbare::check_session(&conn, session_id)
            .map(|user_sess| Some(user_sess))
            .or_else(|e| match e.kind() {
                &ErrorKind::BadSessId => Ok(None),
                &ErrorKind::NoSuchSess => Ok(None),
                _ => Err(e),
            })
    } else {
        Ok(None)
    }
}

pub trait ResponseExt {
    fn refresh_cookie(self, &PgConnection, &Session, IpAddr) -> Self;
    fn expire_cookie(self) -> Self;
}

impl ResponseExt for Response {

    fn refresh_cookie(mut self, conn: &PgConnection, old_sess : &Session, ip: IpAddr) -> Self {
        let sess = ganbare::refresh_session(&conn, &old_sess, ip).expect("Session should already checked to be valid");
    
        let mut cookie = CookiePair::new("session_id".to_owned(), ganbare::sess_to_hex(&sess));
        cookie.path = Some("/".to_owned());
        cookie.domain = Some(SITE_DOMAIN.to_owned());
        cookie.expires = Some(time::now_utc() + time::Duration::weeks(2));
        self.set_cookie(SetCookie(vec![cookie]));
        self
    }
    
    fn expire_cookie(mut self) -> Self {
        let mut cookie = CookiePair::new("session_id".to_owned(), "".to_owned());
        cookie.path = Some("/".to_owned());
        cookie.domain = Some(SITE_DOMAIN.to_owned());
        cookie.expires = Some(time::at_utc(time::Timespec::new(0, 0)));
        self.set_cookie(SetCookie(vec![cookie]));
        self
    }
}


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}

pub fn internal_error<T: std::fmt::Debug>(err: T) -> PencilError {
    error!("{:?}", err);
    PencilError::PenHTTPError(pencil::http_errors::HTTPError::InternalServerError)
}

pub fn bad_request<T: ToString>(err_msg: T) -> Response {
        let body = err_msg.to_string();
        let mut resp = pencil::Response::new(body);
        resp.status_code = 400;
        resp
}

pub trait ResultExt<T> {
    fn err_500(self) -> StdResult<T, PencilError>;
}

impl<T, E: std::fmt::Debug> ResultExt<T> for StdResult<T, E> {
    fn err_500(self) -> StdResult<T, PencilError> {
        self.map_err(|e| internal_error(e))
    }
}

pub trait CarrierInternal<T, E> where E: std::fmt::Debug {
    fn ok_or(self) -> std::result::Result<T, E>;
}

impl<T> CarrierInternal<T, ()> for Option<T> {
    fn ok_or(self) -> std::result::Result<T, ()> {
        match self {
            Some(a) => Ok(a),
            None => Err(()),
        }
    }
}
impl<T, E> CarrierInternal<T, E> for std::result::Result<T, E> where E: std::fmt::Debug {
    fn ok_or(self) -> std::result::Result<T, E> {
        match self {
            Ok(a) => Ok(a),
            Err(e) => Err(e),
        }
    }
}

macro_rules! err_400 {
    ($t:expr , $format_string:expr $(, $param:expr)* ) => { match CarrierInternal::ok_or($t) {
        Ok(a) => { a },
        Err(e) => {
            return Ok(bad_request(
                format!(concat!("<h1>HTTP 400 Bad Request {:?}: ", $format_string, "</h1>"), e $(, $param)*)
            ))
        },
    } }
}

#[cfg(debug_assertions)]
macro_rules! include_templates(
    ($app:ident, $temp_dir:expr, $($file:expr),*) => { {
        $app.template_folder = $temp_dir.to_string();
        $(
            $app.register_template($file);
        )*
        info!("Templates loaded.");
    } }
);

#[cfg(not(debug_assertions))]
macro_rules! include_templates(
    ($app:ident, $temp_dir:expr, $($file:expr),*) => { {
        let mut reg = $app.handlebars_registry.write().expect("This is supposed to fail fast and hard.");
        $(
        reg.register_template_string($file, include_str!(concat!(env!("PWD"), "/", $temp_dir, "/", $file)).to_string())
        .expect("This is supposed to fail fast and hard.");
        )*
    } }
);


pub fn auth_user(req: &mut Request, required_group: &str)
    -> StdResult<(PgConnection, User, Session), PencilError>
{
    match try_auth_user(req)? {
        Some((conn, user, sess)) => {
            if ganbare::check_user_group(&conn, &user, required_group).err_500()? {
                Ok((conn, user, sess))
            } else {
                Err(abort(401).unwrap_err()) // User doesn't belong in the required groups
            }
        },
        None => {
            Err(abort(401).unwrap_err()) // User isn't logged in
        },
    }

}

pub fn try_auth_user(req: &mut Request)
    -> StdResult<Option<(PgConnection, User, Session)>, PencilError> {

    let conn = db_connect().err_500()?;

    if let Some((user, sess)) = get_user(&conn, req).err_500()?
    { // User is logged in

        Ok(Some((conn, user, sess)))

    } else { // Not logged in
        Ok(None)
    }

}

pub fn check_env_vars() { &*DATABASE_URL; &*EMAIL_SERVER; &*SITE_DOMAIN; }

pub fn do_login(email : &str, plaintext_pw : &str, ip : IpAddr) -> Result<Option<(User, Session)>> {
    let conn = db_connect().err_500()?;
    let user = try_or!(ganbare::auth_user(&conn, email, plaintext_pw, &*RUNTIME_PEPPER).err_500()?,
            else return Ok(None));

    let sess = ganbare::start_session(&conn, &user, ip).err_500()?;

    Ok(Some((user, sess)))
}


macro_rules! parse {
    ($expression:expr) => {$expression.map(String::to_string).ok_or(ErrorKind::FormParseError.to_err())?;}
}


}

pub use helpers::*;

pub use std::result::Result as StdResult;
pub use pencil::{Request, PencilResult, PencilError};

pub use ganbare::PgConnection;
pub use ganbare::models::{User, Session};
pub use ganbare::errors::ErrorKind::Msg as ErrMsg;
pub use ganbare::errors::Result as Result;
pub use ganbare::errors::{Error, ErrorKind};

















mod app_pages {

use super::*;
use pencil::redirect;

pub fn hello(req: &mut Request) -> PencilResult {

    if let Some((conn, user, sess)) = try_auth_user(req).err_500()? {

        if let Some(event_redirect) = dispatch_events(req, &conn, &user, &sess)? {
            event_redirect
        } else {
            main_quiz(req, &conn, &user)
        }

    } else {
        return redirect("/login", 303)
    }
}

pub fn ok(req: &mut Request) -> PencilResult {

    let (conn, user, sess) = auth_user(req, "")?;

    let event_name = err_400!(req.form_mut().take("event_ok"), "Field event_ok is missing!");
    let _ = err_400!(ganbare::set_event_done(&conn, &event_name, &user).err_500()?, "Event \"{}\" doesn't exist!", &event_name);


    redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

pub fn dispatch_events(req: &mut Request, conn: &PgConnection, user: &User, sess: &Session)
    -> StdResult<Option<PencilResult>, PencilError> {

    let event_redirect = if ! ganbare::is_event_done(conn, "welcome", &user).err_500()? {

        Some(redirect("/welcome", 303))

    } else if ! ganbare::is_event_done(conn, "survey", &user).err_500()? {

        Some(redirect("/survey", 303))

    } else { None };

    Ok(event_redirect.map(|redirect| redirect.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))))
}

pub fn main_quiz(req: &mut Request, _: &PgConnection, _: &User) -> PencilResult { 
    let context = new_template_context();
    req.app.render_template("main.html", &context)
}

pub fn survey(req: &mut Request) -> PencilResult {
    let (conn, user, _) = auth_user(req, "")?;
    ganbare::initiate_event(&conn, "survey", &user).err_500()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), "survey".into());
    req.app.render_template("survey.html", &context)
}

pub fn welcome(req: &mut Request) -> PencilResult { 
    let (conn, user, _) = auth_user(req, "")?;
    ganbare::initiate_event(&conn, "welcome", &user).err_500()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), "welcome".into());
    req.app.render_template("welcome.html", &context)
}

pub fn login_form(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some(_) = get_user(&conn, req).err_500()? {
        return redirect("/", 303)
    }
    if ! ganbare::is_installed(&conn).err_500()? {
        return redirect("/fresh_install", 303)
    }

    let context = new_template_context();

    req.app.render_template("hello.html", &context)
}

pub fn login_post(request: &mut Request) -> PencilResult {
    let app = request.app;
    let ip = request.request.remote_addr.ip();
    let login_form = request.form_mut();
    let email = login_form.take("email").unwrap_or_default();
    let plaintext_pw = login_form.take("password").unwrap_or_default();

    let conn = db_connect().err_500()?;

    match do_login(&email, &plaintext_pw, ip).err_500()? {
        Some((_, sess)) => {
            redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, ip) )
        },
        None => {
            warn!("Failed login.");
            let mut context = new_template_context();
            context.insert("authError".to_string(), "true".to_string());
            let result = app.render_template("hello.html", &context);
            result.map(|mut resp| {resp.status_code = 401; resp})
        },
    }
}

pub fn logout(request: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some(session_id) = request.cookies().and_then(get_cookie) {
        ganbare::end_session(&conn, &session_id).err_500()?;
    };

    redirect("/", 303).map(ResponseExt::expire_cookie)
}


pub fn confirm_form(request: &mut Request) -> PencilResult {

    let secret = err_400!(request.args().get("secret"), "secret");
    let conn = db_connect()
        .map_err(|e| internal_error(e) )?;
    let (email, _) = ganbare::check_pending_email_confirm(&conn, &secret)
        .err_500()?;

    let mut context = new_template_context();
    context.insert("email".to_string(), email);
    context.insert("secret".to_string(), secret.clone());

    request.app.render_template("confirm.html", &context)
}

pub fn confirm_post(req: &mut Request) -> PencilResult {
    req.load_form_data();
    let ip = req.request.remote_addr.ip();
    let conn = db_connect()
        .err_500()?;
    let secret = err_400!(req.args().get("secret"), "secret missing").clone();
    let password = err_400!(req.form().expect("form data loaded.").get("password"), "password missing");
    let user = match ganbare::complete_pending_email_confirm(&conn, &password, &secret, &*RUNTIME_PEPPER) {
        Ok(u) => u,
        Err(e) => match e.kind() {
            &ganbare::errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
            &ganbare::errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
            _ => return Err(internal_error(e)),
        }
    };

    match do_login(&user.email, &password, ip).err_500()? {
        Some((_, sess)) => {
            redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, ip) )
        },
        None => { Err(internal_error(Error::from(ErrMsg("We just added the user, yet we can't login them in. A bug?".to_string())))) },
    }
}


pub fn change_password_form(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let mut context = new_template_context();

    let password_changed = req.args_mut().take("password_changed")
        .and_then(|a| if a == "true" { Some(a) } else { None })
        .unwrap_or_else(|| "false".to_string());

    context.insert("password_changed".to_string(), password_changed);

    req.app.render_template("change_password.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn change_password(req: &mut Request) -> PencilResult {

    fn parse_form(req: &mut Request) -> Result<(String, String)> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");

        let old_password = parse!(form.get("old_password"));
        let new_password = parse!(form.get("new_password"));
        if new_password != parse!(form.get("new_password_check")) {
            return Err("New passwords don't match!".into());
        }

        Ok((old_password, new_password))
    }

    let (conn, user, sess) = auth_user(req, "")?;

    let (old_password, new_password) = err_400!(parse_form(req), "invalid form data");

    match ganbare::auth_user(&conn, &user.email, &old_password, &*RUNTIME_PEPPER) {
        Err(e) => return match e.kind() {
            &ErrorKind::AuthError => {
                let mut context = new_template_context();
                context.insert("authError".to_string(), "true".to_string());

                req.app.render_template("change_password.html", &context)
                    .map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => Err(internal_error(e)),
        },
        Ok(_) => {
            match ganbare::change_password(&conn, user.id, &new_password, &*RUNTIME_PEPPER) {
                Err(e) => match e.kind() {
                    &ganbare::errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
                    &ganbare::errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
                    _ => return Err(internal_error(e)),
                },
                _ => (),
            };
        },
    };

    redirect("/change_password?password_changed=true", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}


}








































mod manager_pages {

use super::*;
use pencil::redirect;
use pencil::abort;


pub fn fresh_install_form(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect(&*DATABASE_URL).err_500()?;
    if ganbare::is_installed(&conn).err_500()? { return abort(401) };
    let context = new_template_context();
    req.app.render_template("fresh_install.html", &context)
}

pub fn fresh_install_post(req: &mut Request) -> PencilResult {
    let form = req.form_mut();
    let email = err_400!(form.take("email"), "email missing");
    let new_password = err_400!(form.take("new_password"), "new_password missing");
    let new_password_check = err_400!(form.take("new_password_check"), "new_password_check missing");
    if new_password != new_password_check { return Ok(bad_request("passwords don't match")) };

    let conn = ganbare::db_connect(&*DATABASE_URL).err_500()?;
    if ganbare::is_installed(&conn).err_500()? { return abort(401) };

    let user = ganbare::add_user(&conn, &email, &new_password, &*RUNTIME_PEPPER).err_500()?;
    ganbare::join_user_group_by_name(&conn, &user, "admins").err_500()?;
    ganbare::join_user_group_by_name(&conn, &user, "editors").err_500()?;

    let mut context = new_template_context();
    context.insert("install_success".into(), "success".into());
    req.app.render_template("fresh_install.html", &context)
}

pub fn manage(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;

    let (user, sess) = get_user(&conn, req).err_500()?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors").err_500()?
        { return abort(401); }

    let context = new_template_context();

    req.app.render_template("manage.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn add_quiz_form(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    let context = new_template_context();

    req.app.render_template("add_quiz.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn add_quiz_post(req: &mut Request) -> PencilResult  {

    fn parse_form(req: &mut Request) -> Result<(ganbare::NewQuestion, Vec<ganbare::Fieldset>)> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let files = req.files().expect("Form data should be loaded!");;

        let lowest_fieldset = str::parse::<i32>(&parse!(form.get("lowest_fieldset")))?;
        if lowest_fieldset > 10 { return Err(ErrorKind::FormParseError.to_err()); }

        let q_name = parse!(form.get("name"));
        let q_explanation = parse!(form.get("explanation"));
        let question_text = parse!(form.get("question_text"));
        let skill_nugget = parse!(form.get("skill_nugget"));

        let mut fieldsets = Vec::with_capacity(lowest_fieldset as usize);
        for i in 1...lowest_fieldset {

            let q_variations = str::parse::<i32>(&parse!(form.get(&format!("choice_{}_q_variations", i))))?;
            if lowest_fieldset > 100 { return Err(ErrorKind::FormParseError.to_err()); }

            let mut q_variants = Vec::with_capacity(q_variations as usize);
            for v in 1...q_variations {
                if let Some(file) = files.get(&format!("choice_{}_q_variant_{}", i, v)) {
                    if file.size.expect("Size should've been parsed at this phase.") == 0 {
                        continue; // Don't save files with size 0;
                    }
                    let mut file = file.clone();
                    file.do_not_delete_on_drop();
                    q_variants.push(
                        (file.path.clone(),
                        file.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                        file.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                    );
                }
            }
            let answer_audio = files.get(&format!("choice_{}_answer_audio", i));
            let answer_audio_path;
            if let Some(path) = answer_audio {
                if path.size.expect("Size should've been parsed at this phase.") == 0 {
                    answer_audio_path = None;
                } else {
                    let mut cloned_path = path.clone();
                    cloned_path.do_not_delete_on_drop();
                    answer_audio_path = Some(
                        (cloned_path.path.clone(),
                        cloned_path.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                        cloned_path.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                    )
                }
            } else {
                answer_audio_path = None;
            };

            let answer_text = parse!(form.get(&format!("choice_{}_answer_text", i)));
            let fields = ganbare::Fieldset {q_variants: q_variants, answer_audio: answer_audio_path, answer_text: answer_text};
            fieldsets.push(fields);
        }

        Ok((ganbare::NewQuestion{q_name, q_explanation, question_text, skill_nugget}, fieldsets))
    }

    let (conn, _, sess) = auth_user(req, "editors")?;

    let form = parse_form(&mut *req).map_err(|ee| { error!("{:?}", ee); abort(400).unwrap_err()})?;
    let result = ganbare::create_quiz(&conn, form.0, form.1);
    result.map_err(|e| match e.kind() {
        &ErrorKind::FormParseError => abort(400).unwrap_err(),
        _ => abort(500).unwrap_err(),
    })?;

    redirect("/add_quiz", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

pub fn add_word_form(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let context = new_template_context();

    req.app.render_template("add_word.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn add_word_post(req: &mut Request) -> PencilResult  {

    fn parse_form(req: &mut Request) -> Result<ganbare::NewWordFromStrings> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let uploaded_files = req.files().expect("Form data should be loaded!");

        let num_variants = str::parse::<i32>(&parse!(form.get("audio_variations")))?;
        if num_variants > 20 { return Err(ErrorKind::FormParseError.to_err()); }

        let word = parse!(form.get("word"));
        let explanation = parse!(form.get("explanation"));
        let nugget = parse!(form.get("skill_nugget"));

        let mut files = Vec::with_capacity(num_variants as usize);
        for v in 1...num_variants {
            if let Some(file) = uploaded_files.get(&format!("audio_variant_{}", v)) {
                if file.size.expect("Size should've been parsed at this phase.") == 0 {
                    continue; // Don't save files with size 0;
                }
                let mut file = file.clone();
                file.do_not_delete_on_drop();
                files.push(
                    (file.path.clone(),
                    file.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                    file.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                );
            }
        }

        Ok(ganbare::NewWordFromStrings{word, explanation, narrator: "".into(), nugget, files})
    }

    let (conn, _, sess) = auth_user(req, "editors")?;

    let word = parse_form(req)
            .map_err(|_| abort(400).unwrap_err())?;

    ganbare::create_word(&conn, word).err_500()?;
    
    redirect("/add_word", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

pub fn add_users_form(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "admins")?;

    let context = new_template_context();
    req.app.render_template("add_users.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn add_users(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "admins")?;

    req.load_form_data();
    let form = req.form().expect("The form data is loaded.");
    let emails = err_400!(form.get("emailList"),"emailList missing?");
    for row in emails.split("\n") {
        let mut fields = row.split_whitespace();
        let email = err_400!(fields.next(), "email field missing?");
        let mut groups = vec![];
        for field in fields {
            groups.push(try_or!(ganbare::get_group(&conn, &field.to_lowercase())
                .err_500()?, else return abort(400)).id);
        }
        let secret = ganbare::add_pending_email_confirm(&conn, email, groups.as_ref())
            .err_500()?;
        ganbare::email::send_confirmation(email, &secret, &*EMAIL_SERVER, &*EMAIL_DOMAIN, &*SITE_DOMAIN, &**req.app.handlebars_registry.read()
                .expect("The registry is basically read-only after startup."))
            .err_500()?;
    }

    let context = new_template_context();
    req.app.render_template("add_users.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


}





























mod http_api {

use super::*;
use pencil::{abort, jsonify, Response, redirect};
use chrono;
use rand;
use pencil::helpers::{send_file, send_from_directory};
use rustc_serialize;
use regex;
use unicode_normalization::UnicodeNormalization;

pub fn get_audio(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let line_id = req.view_args.get("line_id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let line_id = line_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let (file_name, mime_type) = ganbare::get_audio_file(&conn, line_id)
        .map_err(|e| {
            match e.kind() {
                &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                _ => abort(500).unwrap_err(),
            }
        })?;

    use pencil::{PencilError, HTTPError};

    let file_path = AUDIO_DIR.to_string() + "/" + &file_name;

    send_file(&file_path, mime_type, false)
        .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Audio file not found? The audio file database/folder is borked? {}", file_path); internal_error(e) },
            _ => { internal_error(e) }
        })
}

pub fn get_image(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let file_name = req.view_args.get("filename").expect("Pencil guarantees that filename should exist as an arg.");

    use pencil::{PencilError, HTTPError};

    send_from_directory(&*IMAGES_DIR, &file_name, false)
        .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Image file not found! {}", file_name); e },
            _ => { internal_error(e) }
        })
}

#[derive(RustcEncodable)]
struct QuizJson {
    quiz_type: String,
    question_id: i32,
    explanation: String,
    question: (String, i32),
    right_a: i32,
    answers: Vec<(i32, String, Option<i32>)>,
    due_delay: i32,
    due_date: Option<String>,
}

#[derive(RustcEncodable)]
struct WordJson {
    show_accents: bool,
    quiz_type: String,
    id: i32,
    word: String,
    explanation: String,
    audio_id: i32,
    due_delay: i32,
    due_date: Option<String>,
}

pub fn card_to_json(card: ganbare::Card) -> PencilResult {
    use rand::Rng;
    use ganbare::Card::*;
    let mut rng = rand::thread_rng();
    match card {
    Quiz(ganbare::Quiz{ question, question_audio, right_answer_id, answers, due_delay, due_date }) => {

        let mut answers_json = Vec::with_capacity(answers.len());

        let chosen_q_audio = rng.choose(&question_audio).expect("Audio for a Question: Shouldn't be empty! Borked database?");
        

        for a in answers {
            answers_json.push((a.id, a.answer_text, a.a_audio_bundle));
        }

        rng.shuffle(&mut answers_json);
        

        jsonify(&QuizJson {
            quiz_type: "question".into(),
            question_id: question.id,
            explanation: question.q_explanation,
            question: (question.question_text, chosen_q_audio.id),
            right_a: right_answer_id,
            answers: answers_json,
            due_delay,
            due_date: due_date.map(|d| d.to_rfc3339()),
        })
    },
    Word((ganbare::models::Word { id, word, explanation, .. }, audio_files)) => {

        let chosen_audio = rng.choose(&audio_files).expect("Audio for a Word: Shouldn't be empty! Borked database?");

        jsonify(&WordJson {
            show_accents: true, // FIXME
            quiz_type: "word".into(),
            id,
            word: word.nfc().collect::<String>(), // Unicode normalization, because "word" is going to be accented kana-by-kana.
            explanation,
            audio_id: chosen_audio.id,
            due_delay: 30, // FIXME
            due_date: Some(chrono::UTC::now()).map(|d| d.to_rfc3339()), // FIXME
        })
    },
    }
}

pub fn new_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    let new_quiz = ganbare::get_new_quiz(&conn, &user).err_500()?;

    let card = try_or!{new_quiz, else return jsonify(&())}; 

    card_to_json(card).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn next_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    fn parse_answer(req : &mut Request) -> Result<ganbare::Answered> {
        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let answer_type = &parse!(form.get("type"));

        if answer_type == "word" {
            let word_id = str::parse::<i32>(&parse!(form.get("word_id")))?;
            let times_audio_played = str::parse::<i32>(&parse!(form.get("times_audio_played")))?;
            let time = str::parse::<i32>(&parse!(form.get("time")))?;
            Ok(ganbare::Answered::Word(
                ganbare::AnsweredWord{word_id, times_audio_played, time}
            ))
        } else if answer_type == "question" {
            let question_id = str::parse::<i32>(&parse!(form.get("question_id")))?;
            let right_answer_id = str::parse::<i32>(&parse!(form.get("right_a_id")))?;
            let answered_id = str::parse::<i32>(&parse!(form.get("answered_id")))?;
            let answered_id = if answered_id > 0 { Some(answered_id) } else { None }; // Negatives mean that question was unanswered (due to time limit)
            let q_audio_id = str::parse::<i32>(&parse!(form.get("q_audio_id")))?;
            let active_answer_time = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let full_answer_time = str::parse::<i32>(&parse!(form.get("full_answer_time")))?;
            Ok(ganbare::Answered::Question(
                ganbare::AnsweredQuestion{question_id, right_answer_id, answered_id, q_audio_id, active_answer_time, full_answer_time}
            ))
        } else {
            Err(ErrorKind::FormParseError.into())
        }
    };

    let answer = parse_answer(req)
        .map_err(|_| abort(400).unwrap_err())?;

    let new_quiz = ganbare::get_next_quiz(&conn, &user, answer)
        .err_500()?;

    let card = try_or!{new_quiz, else return jsonify(&())}; 
    card_to_json(card).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn get_item(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_word" => {
            let item = ganbare::get_word(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
                },
        "get_question" => {
            let item = ganbare::get_question(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn get_all(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_nuggets" => {
            let items = ganbare::get_skill_nuggets(&conn).err_500()?;
            jsonify(&items)
        },
        "get_bundles" => {
            let items = ganbare::get_audio_bundles(&conn).err_500()?;
            jsonify(&items)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn set_published(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    match endpoint.as_ref() {
        "publish_words" => {
            ganbare::publish_word(&conn, id, true).err_500()?;
        },
        "publish_questions" => {
            ganbare::publish_question(&conn, id, true).err_500()?;
        },
        "unpublish_words" => {
            ganbare::publish_word(&conn, id, false).err_500()?;
        },
        "unpublish_questions" => {
            ganbare::publish_question(&conn, id, false).err_500()?;
        },
        _ => {
            return abort(500)
        },
    };
    let mut resp = Response::new_empty();
    resp.status_code = 204;
    Ok(resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn update_item(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.")
                .parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    lazy_static! {
        // Taking JSON encoding into account: " is escaped as \"
        static ref RE: regex::Regex = regex::Regex::new(r###"<img ([^>]* )?src=\\"(?P<src>[^"]*)\\"( [^>]*)?>"###).unwrap();
    }
    let text = RE.replace_all(&text, r###"<img src=\"$src\">"###);

    let json;
    match endpoint.as_str() {
        "update_word" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_word(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);

        },
        "update_question" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_question(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_answer" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_answer(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        _ => return abort(500),
    }
    
    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn post_question(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    use ganbare::models::{UpdateQuestion, UpdateAnswer, NewQuizQuestion, NewAnswer};

    let (qq, aas) : (UpdateQuestion, Vec<UpdateAnswer>) = rustc_serialize::json::decode(&text)
            .map_err(|_| abort(400).unwrap_err())?;

    fn parse_qq(qq: &UpdateQuestion) -> Result<NewQuizQuestion> {
        let qq = NewQuizQuestion {
            skill_id: qq.skill_id.ok_or(ErrorKind::FormParseError.to_err())?,
            q_name: qq.q_name.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            q_explanation: qq.q_explanation.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            question_text: qq.question_text.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            skill_level: qq.skill_level.ok_or(ErrorKind::FormParseError.to_err())?,
        };
        Ok(qq)
    }

    fn parse_aa(aa: &UpdateAnswer) -> Result<NewAnswer> {
        let aa = NewAnswer {
            question_id: aa.question_id.ok_or(ErrorKind::FormParseError.to_err())?,
            a_audio_bundle: aa.a_audio_bundle.unwrap_or(None),
            q_audio_bundle: aa.q_audio_bundle.ok_or(ErrorKind::FormParseError.to_err())?,
            answer_text: aa.answer_text.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
        };
        Ok(aa)
    }

    let new_qq = parse_qq(&qq)
            .map_err(|_| abort(400).unwrap_err())?;

    let mut new_aas = vec![];
    for aa in &aas {
        let new_aa = parse_aa(aa)
            .map_err(|_| abort(400).unwrap_err())?;
        new_aas.push(new_aa);
    }

    let id = ganbare::post_question(&conn, new_qq, new_aas).err_500()?;
        
    let new_url = format!("/api/questions/{}", id);

    redirect(&new_url, 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}


}































pub fn main() {
    env_logger::init().unwrap();
    info!("Starting.");
    check_env_vars();
    let conn = ganbare::db_connect(&*DATABASE_URL).expect("Can't connect to database!");
    ganbare::check_db(&conn).expect("Something funny with the DB!");
    info!("Database OK.");

    let mut app = Pencil::new(".");
   
    include_templates!(app, "templates", "base.html", "fresh_install.html", "welcome.html",
        "hello.html", "main.html", "confirm.html", "add_quiz.html", "add_word.html", "survey.html",
        "manage.html", "change_password.html", "add_users.html", "email_confirm_email.html");
    
    app.enable_static_file_handling();

    // BASIC FUNCTIONALITY
    app.get("/", "hello", app_pages::hello);
    app.get("/welcome", "welcome", app_pages::welcome);
    app.get("/survey", "survey", app_pages::survey);
    app.post("/ok", "ok", app_pages::ok);
    app.get("/login", "login_form", app_pages::login_form);
    app.post("/login", "login_post", app_pages::login_post);
    app.post("/logout", "logout", app_pages::logout);
    app.get("/confirm", "confirm_form", app_pages::confirm_form);
    app.post("/confirm", "confirm_post", app_pages::confirm_post);
    app.get("/change_password", "change_password_form", app_pages::change_password_form);
    app.post("/change_password", "change_password", app_pages::change_password);

    // MANAGER PAGES
    app.get("/fresh_install", "fresh_install_form", manager_pages::fresh_install_form);
    app.post("/fresh_install", "fresh_install_post", manager_pages::fresh_install_post);
    app.get("/add_quiz", "add_quiz_form", manager_pages::add_quiz_form);
    app.post("/add_quiz", "add_quiz_post", manager_pages::add_quiz_post);
    app.get("/add_users", "add_users_form", manager_pages::add_users_form);
    app.post("/add_users", "add_users", manager_pages::add_users);
    app.get("/add_word", "add_word_form", manager_pages::add_word_form);
    app.post("/add_word", "add_word_post", manager_pages::add_word_post);
    app.get("/manage", "manage", manager_pages::manage);

    // HTTP API
    app.get("/api/nuggets", "get_nuggets", http_api::get_all);
    app.get("/api/bundles", "get_bundles", http_api::get_all);
    app.get("/api/questions/<id:int>", "get_question", http_api::get_item);
    app.get("/api/words/<id:int>", "get_word", http_api::get_item);
    app.put("/api/questions/<id:int>?publish", "publish_questions", http_api::set_published);
    app.post("/api/question", "post_question", http_api::post_question);
    app.put("/api/words/<id:int>?publish", "publish_words", http_api::set_published);
    app.put("/api/questions/<id:int>?unpublish", "unpublish_questions", http_api::set_published);
    app.put("/api/words/<id:int>?unpublish", "unpublish_words", http_api::set_published);
    app.put("/api/words/<id:int>", "update_word", http_api::update_item);
    app.put("/api/questions/<id:int>", "update_question", http_api::update_item);
    app.put("/api/questions/answers/<id:int>", "update_answer", http_api::update_item);
    app.get("/api/new_quiz", "new_quiz", http_api::new_quiz);
    app.post("/api/next_quiz", "next_quiz", http_api::next_quiz);
    app.get("/api/audio/<line_id:int>", "get_audio", http_api::get_audio);
    app.get("/api/images/<filename:string>", "get_image", http_api::get_image);


    info!("Ready. Running on {}, serving at {}", *SERVER_BINDING, *SITE_DOMAIN);
    app.run(*SERVER_BINDING);
}
