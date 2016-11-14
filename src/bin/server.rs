#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]

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
extern crate rand;
extern crate chrono;

use dotenv::dotenv;
use std::env;
use ganbare::errors::*;
use std::net::IpAddr;

use mime::Mime;
use std::path::PathBuf;
use std::collections::BTreeMap;
use hyper::header::{SetCookie, CookiePair, Cookie};
use pencil::{Pencil, Request, Response, PencilResult, redirect, abort, jsonify};
use pencil::helpers::send_file;
use ganbare::models::{User, Session};


//const JQUERY_URL: &'static str = "https://ajax.googleapis.com/ajax/libs/jquery/3.1.0/jquery.min.js";
const JQUERY_URL: &'static str = "/static/assets/js/jquery.min.js";

macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}


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

fn get_user(conn : &ganbare::PgConnection, req : &Request) -> Result<Option<(User, Session)>> {
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

trait ResponseExt {
    fn refresh_cookie(self, &ganbare::PgConnection, &Session, IpAddr) -> Self;
    fn expire_cookie(self) -> Self;
}

impl ResponseExt for Response {

fn refresh_cookie(mut self, conn: &ganbare::PgConnection, old_sess : &Session, ip: IpAddr) -> Self {
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

fn hello(request: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*request).map_err(|_| abort(500).unwrap_err())?;

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    context.insert("jquery_url".to_string(), JQUERY_URL.to_string());

    match user_session {
        Some((_, sess)) => request.app.render_template("main.html", &context)
                            .map(|resp| resp.refresh_cookie(&conn, &sess, request.remote_addr().ip())),
        None => request.app.render_template("hello.html", &context),
    }
}

fn login(request: &mut Request) -> PencilResult {
    let app = request.app;
    let ip = request.request.remote_addr.ip();
    let login_form = request.form_mut();
    let email = login_form.take("email").unwrap_or_default();
    let plaintext_pw = login_form.take("password").unwrap_or_default();

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    context.insert("authError".to_string(), "true".to_string());

    do_login(&email, &plaintext_pw, ip)
        .or_else(|e| match e {
            pencil::PencilError::PenHTTPError(pencil::HTTPError::Unauthorized) => {
                let result = app.render_template("hello.html", &context);
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

    redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &session, ip) )
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
    let password = request.form_mut().get("password")
        .ok_or_else(|| abort(500).unwrap_err() )?;
    let user = ganbare::complete_pending_email_confirm(&conn, password, &secret).map_err(|_| abort(500).unwrap_err())?;

    do_login(&user.email, &password, ip)
}

macro_rules! parse {
    ($expression:expr) => {$expression.map(String::to_string).ok_or(ErrorKind::FormParseError.to_err())?;}
}

fn internal_error<T: std::error::Error>(err: T) -> pencil::PencilError {
    println!("{:?}", err);
    abort(500).unwrap_err()
}

fn add_quiz_form(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    req.app.render_template("add_quiz.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn add_quiz_post(req: &mut Request) -> PencilResult  {

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

    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*req).map_err(|_| abort(500).unwrap_err())?;

    let (_, sess) = try_or!{user_session, else return abort(401)};

    let form = parse_form(&mut *req).map_err(|ee| { println!("Error: {:?}", ee); abort(400).unwrap_err()})?;
    let result = ganbare::create_quiz(&conn, form.0, form.1);
    result.map_err(|e| match e.kind() {
        &ErrorKind::FormParseError => abort(400).unwrap_err(),
        _ => abort(500).unwrap_err(),
    })?;

    redirect("/add_quiz", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

fn add_word_form(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let mut context = BTreeMap::new();
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    req.app.render_template("add_word.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}
fn add_word_post(req: &mut Request) -> PencilResult  {

    fn parse_form(req: &mut Request) -> Result<(String, String, String, Vec<(PathBuf, Option<String>, Mime)>)> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let files = req.files().expect("Form data should be loaded!");

        let num_variants = str::parse::<i32>(&parse!(form.get("audio_variations")))?;
        if num_variants > 20 { return Err(ErrorKind::FormParseError.to_err()); }

        let word = parse!(form.get("word"));
        let explanation = parse!(form.get("explanation"));
        let skill_nugget = parse!(form.get("skill_nugget"));

        let mut audio_variants = Vec::with_capacity(num_variants as usize);
        for v in 1...num_variants {
            if let Some(file) = files.get(&format!("audio_variant_{}", v)) {
                if file.size.expect("Size should've been parsed at this phase.") == 0 {
                    continue; // Don't save files with size 0;
                }
                let mut file = file.clone();
                file.do_not_delete_on_drop();
                audio_variants.push(
                    (file.path.clone(),
                    file.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                    file.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                );
            }
        }

        Ok((word, explanation, skill_nugget, audio_variants))
    }

    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*req).map_err(|_| abort(500).unwrap_err())?;

    let (_, sess) = try_or!{user_session, else return abort(401)};

    let (word, explanation, skill_nugget, audio) = parse_form(req)
            .map_err(|_| abort(400).unwrap_err())?;

    ganbare::create_word(&conn, (word, explanation, skill_nugget, audio))
        .map_err(|_| abort(500).unwrap_err())?;
    
    redirect("/add_word", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}


fn get_line(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let line_id = req.view_args.get("line_id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let line_id = line_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let (file_path, mime_type) = ganbare::get_line_file(&conn, line_id)
        .map_err(|e| {
            match e.kind() {
                &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                _ => abort(500).unwrap_err(),
            }
        })?;

    use pencil::{PencilError, HTTPError};

    send_file(&file_path, mime_type, false)
        .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { println!("The file database is borked?"); internal_error(e) },
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

fn card_to_json(card: ganbare::Card) -> PencilResult {
    use rand::Rng;
    use ganbare::Card::*;
    let mut rng = rand::thread_rng();
    match card {
    Quiz(ganbare::Quiz{ question, question_audio, right_answer_id, answers, due_delay, due_date }) => {

        let mut answers_json = Vec::with_capacity(answers.len());

        let chosen_q_audio = rng.choose(&question_audio).expect("Shouldn't be empty!");
        

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

        let chosen_audio = rng.choose(&audio_files).expect("Shouldn't be empty!");

        jsonify(&WordJson {
            show_accents: true, // FIXME
            quiz_type: "word".into(),
            id,
            word,
            explanation,
            audio_id: chosen_audio.id,
            due_delay: 30, // FIXME
            due_date: Some(chrono::UTC::now()).map(|d| d.to_rfc3339()), // FIXME
        })
    },
    }
}

fn new_quiz(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err())?; // Unauthorized

    let new_quiz = ganbare::get_new_quiz(&conn, &user)
        .map_err(|_| abort(500).unwrap_err())?;

    let card = try_or!{new_quiz, else return jsonify(&())}; 

    card_to_json(card).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn next_quiz(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err())?; // Unauthorized

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
            let time = str::parse::<i32>(&parse!(form.get("time")))?;
            let due_delay = str::parse::<i32>(&parse!(form.get("due_delay")))?;
            Ok(ganbare::Answered::Question(
                ganbare::AnsweredQuestion{question_id, right_answer_id, answered_id, q_audio_id, due_delay, time}
            ))
        } else {
            Err(ErrorKind::FormParseError.into())
        }
    };

    let answer = parse_answer(req)
        .map_err(|_| abort(400).unwrap_err())?;

    let new_quiz = ganbare::get_next_quiz(&conn, &user, answer)
        .map_err(|e| internal_error(e))?;

    let card = try_or!{new_quiz, else return jsonify(&())}; 
    card_to_json(card).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn change_password_form(req: &mut Request) -> PencilResult {
    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let mut context = BTreeMap::new();

    let password_changed = req.args_mut().take("password_changed")
        .and_then(|a| if a == "true" { Some(a) } else { None })
        .unwrap_or_else(|| "false".to_string());

    context.insert("password_changed".to_string(), password_changed);
    context.insert("title".to_string(), "akusento.ganba.re".to_string());
    req.app.render_template("change_password.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn change_password(req: &mut Request) -> PencilResult {


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

    let conn = ganbare::db_connect()
        .map_err(|_| abort(500).unwrap_err())?;

    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let (old_password, new_password) = parse_form(req)
        .map_err(|_| abort(400).unwrap_err())?;

    match ganbare::auth_user(&conn, &user.email, &old_password) {
        Err(e) => return match e.kind() {
            &ErrorKind::AuthError => {
                let mut context = BTreeMap::new();
                context.insert("title".to_string(), "akusento.ganba.re".to_string());
                context.insert("authError".to_string(), "true".to_string());

                req.app.render_template("change_password.html", &context)
                    .map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => abort(500),
        },
        Ok(_) => {
            ganbare::change_password(&conn, user.id, &new_password)
                .map_err(|e| match e.kind() {
                    &ErrorKind::PasswordTooShort => abort(400).unwrap_err(),
                    _ => internal_error(e),
                })?;
        },
    };

    redirect("/change_password?password_changed=true", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

fn main() {
    dotenv().ok();
    let mut app = Pencil::new(".");
    app.register_template("hello.html");
    app.register_template("main.html");
    app.register_template("confirm.html");
    app.register_template("add_quiz.html");
    app.register_template("add_word.html");
    app.register_template("change_password.html");
    app.enable_static_file_handling();

 //   app.set_debug(true);
 //   app.set_log_level();
 //   env_logger::init().unwrap();
    debug!("* Running on http://localhost:5000/, serving at {:?}", *SITE_DOMAIN);

    app.get("/", "hello", hello);
    app.post("/logout", "logout", logout);
    app.post("/login", "login", login);
    app.get("/confirm", "confirm", confirm);
    app.get("/add_quiz", "add_quiz_form", add_quiz_form);
    app.post("/add_quiz", "add_quiz_post", add_quiz_post);
    app.get("/add_word", "add_word_form", add_word_form);
    app.post("/add_word", "add_word_post", add_word_post);
    app.post("/confirm", "confirm_final", confirm_final);
    app.get("/change_password", "change_password_form", change_password_form);
    app.post("/change_password", "change_password", change_password);
    app.get("/api/new_quiz", "new_quiz", new_quiz);
    app.post("/api/next_quiz", "next_quiz", next_quiz);
    app.get("/api/get_line/<line_id:int>", "get_line", get_line);

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
