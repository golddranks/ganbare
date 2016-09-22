#![feature(question_mark)]
extern crate ganbare;
extern crate pencil;
extern crate dotenv;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
#[macro_use]  extern crate lazy_static;
#[macro_use]  extern crate mime;
extern crate time;
extern crate rustc_serialize;

use dotenv::dotenv;
use std::env;
use ganbare::errors::*;
use std::net::IpAddr;

use std::collections::BTreeMap;
use hyper::header::{SetCookie, CookiePair, Cookie};
use pencil::{Pencil, Request, Response, PencilResult, redirect, abort, jsonify, HTTPError};
use pencil::helpers::send_file;
use ganbare::models::{User, Session};

lazy_static! {
    static ref SITE_DOMAIN : String = { dotenv().ok(); env::var("GANBARE_SITE_DOMAIN")
    .unwrap_or_else(|_| "".into()) };
}


pub fn get_cookie(cookies : &Cookie) -> Option<&str> {
    for c in cookies.0.iter() {
        if c.name == "session_id" {
            return Some(c.value.as_ref());
        }
    };
    None
}

fn get_user_refresh_sessid(conn : &ganbare::PgConnection, req : &Request) -> Result<Option<(User, Session)>> {
    if let Some(session_id) = req.cookies().and_then(get_cookie) {
        ganbare::check_session(&conn, session_id, req.request.remote_addr.ip())
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

trait ResponseExt {
    fn refresh_cookie(mut self, &Session) -> Self;
    fn expire_cookie(mut self) -> Self;
}

impl ResponseExt for Response {

fn refresh_cookie(mut self, sess : &Session) -> Self {
    let mut cookie = CookiePair::new("session_id".to_owned(), ganbare::sess_to_hex(sess));
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

fn hello(request: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user_refresh_sessid(&conn, &*request).map_err(|_| abort(500).unwrap_err())?;

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());

    match user_session {
        Some((_, sess)) => request.app.render_template("main.html", &context)
                            .map(|resp| resp.refresh_cookie(&sess)),
        None => request.app.render_template("hello.html", &context),
    }
}

fn login(request: &mut Request) -> PencilResult {
    let app = request.app;
    let ip = request.request.remote_addr.ip();
    let login_form = request.form();
    let email = login_form.get("email").map(String::to_string).unwrap_or_default();
    let plaintext_pw = login_form.get("password").map(String::to_string).unwrap_or_default();

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    context.insert("authError".to_string(), "true".to_string());

    do_login(&email, &plaintext_pw, ip)
        .or_else(|e| match e {
            pencil::PencilError::PenHTTPError(Unauthorized) => {
                let mut result = app.render_template("hello.html", &context);
                result.map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => Err(e),
        })
}

fn do_login(email : &str, plaintext_pw : &str, ip : IpAddr) -> PencilResult {
    let conn = ganbare::db_connect().map_err(|_| abort(500).unwrap_err())?;
    let user;
    {
        user = ganbare::auth_user(&conn, email, plaintext_pw)
            .map_err(|e| match e.kind() {
                    &ErrorKind::AuthError => abort(401).unwrap_err(),
                    _ => abort(500).unwrap_err(),
                })?;
    };

    let session = ganbare::start_session(&conn, &user, ip)
        .map_err(|_| abort(500).unwrap_err())?;

    redirect("/", 303).map(|resp| resp.refresh_cookie(&session) )
}


fn logout(request: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect().map_err(|_| abort(500).unwrap_err())?;
    if let Some(session_id) = request.cookies().and_then(get_cookie) {
        ganbare::end_session(&conn, &session_id)
            .map_err(|_| abort(500).unwrap_err())?;
    };

    redirect("/", 303).map(ResponseExt::expire_cookie)
}

fn error(err_msg : &str) -> pencil::PencilError {
    println!("Error: {}", err_msg);
    abort(500).unwrap_err()
}


fn confirm(request: &mut Request) -> PencilResult {

    let secret = request.args().get("secret")
        .ok_or_else(|| error("Can't get argument secret from URL!") )?;
    let conn = ganbare::db_connect()
        .map_err(|_| error("Can't connect to database!") )?;
    let email = ganbare::check_pending_email_confirm(&conn, &secret)
        .map_err(|_| error("Check pending email confirms failed!"))?;

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    context.insert("email".to_string(), email);
    context.insert("secret".to_string(), secret.clone());

    request.app.render_template("confirm.html", &context)
}

fn confirm_final(request: &mut Request) -> PencilResult {
    let ip = request.request.remote_addr.ip();
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let secret = request.args().get("secret")
            .ok_or_else(|| abort(500).unwrap_err() )?.clone();
    let password = request.form().get("password")
        .ok_or_else(|| abort(500).unwrap_err() )?;
    let user = ganbare::complete_pending_email_confirm(&conn, password, &secret).map_err(|_| abort(500).unwrap_err())?;

    do_login(&user.email, &password, ip)
}

#[derive(RustcEncodable)]
struct Quiz {
    username: String,
    lines: String,
}

fn new_quiz(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user_refresh_sessid(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err())?; // Unauthorized

    let line_path = "/api/get_line/".to_string() + &ganbare::get_new_quiz(&conn, &user)
        .map_err(|_| abort(500).unwrap_err())?;
 
    jsonify(&Quiz { username: user.email,lines: line_path })
        .map(|resp| resp.refresh_cookie(&sess))
}

fn get_line(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user_refresh_sessid(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let line_id = req.view_args.get("line_id").expect("Line ID should exist as an arg.");
    let (file_path, mime_type) = ganbare::get_line_file(&conn, line_id);

    send_file(&file_path, mime_type, false)
        .map(|resp| resp.refresh_cookie(&sess))
}


fn main() {
    dotenv().ok();
    let mut app = Pencil::new(".");
    app.register_template("hello.html");
    app.register_template("main.html");
    app.register_template("confirm.html");
    app.enable_static_file_handling();

//    app.set_debug(true);
//    app.set_log_level();
//    env_logger::init().unwrap();
    debug!("* Running on http://localhost:5000/, serving at {:?}", *SITE_DOMAIN);

    app.get("/", "hello", hello);
    app.post("/logout", "logout", logout);
    app.post("/login", "login", login);
    app.get("/confirm", "confirm", confirm);
    app.post("/confirm", "confirm_final", confirm_final);
    app.get("/api/new_quiz", "new_quiz", new_quiz);
    app.get("/api/get_line/<line_id:string>", "get_line", get_line);

    let binding = match env::var("GANBARE_SERVER_BINDING") {
        Err(_) => {
            println!("Specify the ip address and port to listen (e.g. 0.0.0.0:80) in envvar GANBARE_SERVER_BINDING!");
            return;
        },
        Ok(ok) => ok,
    };
    println!("Ready to run at {}", binding);
    app.run(binding.as_str());
}
