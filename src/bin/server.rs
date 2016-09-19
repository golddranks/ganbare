#![feature(question_mark)]
extern crate ganbare;
extern crate pencil;
extern crate dotenv;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
#[macro_use]  extern crate lazy_static;
extern crate time;

use dotenv::dotenv;
use std::env;
use ganbare::errors::*;

use std::collections::BTreeMap;
use hyper::header::{SetCookie, CookiePair, Cookie};
use pencil::{Pencil, Request, Response, PencilResult, redirect, abort};
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

fn get_user(req : &Request) -> Result<Option<(User, Session)>> {
    if let Some(session_id) = req.cookies().and_then(get_cookie) {
        let conn = ganbare::db_connect().map_err(|_| abort(500).unwrap_err())?;
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
    let user_session = get_user(&*request).map_err(|_| abort(500).unwrap_err())?;

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());

    match user_session {
        Some((_, sess)) => request.app.render_template("main.html", &context)
                            .map(|resp| resp.refresh_cookie(&sess)),
        None => request.app.render_template("hello.html", &context),
    }
}


fn login(request: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect().map_err(|_| abort(500).unwrap_err())?;
    let user;
    {
        let login_form = request.form();
        let email = login_form.get("email").map(String::as_ref).unwrap_or("");
        let plaintext_pw = login_form.get("password").map(String::as_ref).unwrap_or("");
        user = ganbare::auth_user(&conn, email, plaintext_pw)
            .map_err(|e| match e.kind() {
                    &ErrorKind::AuthError => { println!("VITTU {:?}", e); abort(401).unwrap_err() },
                    _ => abort(500).unwrap_err(),
                })?;
    };

    let session = ganbare::start_session(&conn, &user, request.request.remote_addr.ip())
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


fn main() {
    dotenv().ok();
    let mut app = Pencil::new(".");
    app.register_template("hello.html");
    app.register_template("main.html");
    app.enable_static_file_handling();

//    app.set_debug(true);
//    app.set_log_level();
//    env_logger::init().unwrap();
    debug!("* Running on http://localhost:5000/, serving at {:?}", *SITE_DOMAIN);

    app.get("/", "hello", hello);
    app.get("/logout", "logout", logout);
    app.post("/login", "login", login);

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
