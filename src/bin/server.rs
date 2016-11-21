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
extern crate unicode_normalization;
extern crate regex;

use unicode_normalization::UnicodeNormalization;
use std::env;
use ganbare::errors::*;
use std::net::IpAddr;

use std::collections::BTreeMap;
use hyper::header::{SetCookie, CookiePair, Cookie};
use pencil::{Pencil, Request, Response, PencilResult, redirect, abort, jsonify};
use pencil::helpers::{send_file, send_from_directory};
use ganbare::models::{User, Session};
use rustc_serialize::base64::FromBase64;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::RwLock;


macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}

lazy_static! {

    static ref APP_INSTALLED : RwLock<bool> = RwLock::new(false);

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    static ref SITE_DOMAIN : String = { dotenv::dotenv().ok(); env::var("GANBARE_SITE_DOMAIN")
        .expect("GANBARE_SITE_DOMAIN: Set the site domain! (Without it, the cookies don't work.)") };

    static ref EMAIL_SERVER : SocketAddr = { dotenv::dotenv().ok();
        let binding = env::var("GANBARE_EMAIL_SERVER")
        .expect("GANBARE_EMAIL_SERVER: Specify an outbound email server, like this: mail.yourisp.com:25");
        binding.to_socket_addrs().expect("Format: domain:port").next().expect("Format: domain:port") };

    static ref EMAIL_DOMAIN : String = { dotenv::dotenv().ok(); env::var("GANBARE_EMAIL_DOMAIN")
        .unwrap_or_else(|_|  env::var("GANBARE_SITE_DOMAIN").unwrap_or_else(|_| "".into())) };

    static ref SERVER_BINDING : SocketAddr = { dotenv::dotenv().ok();
        let binding = env::var("GANBARE_SERVER_BINDING")
        .unwrap_or_else(|_| "localhost:8080".into());
        binding.to_socket_addrs().expect("GANBARE_SERVER_BINDING: Format: domain:port").next()
        .expect("GANBARE_SERVER_BINDING: Format: domain:port") };

    static ref JQUERY_URL : String = { dotenv::dotenv().ok(); env::var("GANBARE_JQUERY")
        .unwrap_or_else(|_| "/static/js/jquery.min.js".into()) };

    static ref AUDIO_DIR : String = { dotenv::dotenv().ok(); env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "audio".into()) };

    static ref IMAGES_DIR : String = { dotenv::dotenv().ok(); env::var("GANBARE_IMAGES_DIR")
        .unwrap_or_else(|_| "images".into()) };

    static ref RUNTIME_PEPPER : Vec<u8> = { dotenv::dotenv().ok();
        let pepper = env::var("GANBARE_RUNTIME_PEPPER")
        .expect("Environmental variable GANBARE_RUNTIME_PEPPER must be set! (format: 256-bit random value encoded as base64)")
        .from_base64().expect("Environmental variable GANBARE_RUNTIME_PEPPER isn't valid Base64!");
        if pepper.len() != 32 { panic!("The value must be 256-bit, that is, 32 bytes long!") }; pepper
    };

}

fn db_connect() -> Result<ganbare::PgConnection> {
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

fn new_template_context() -> BTreeMap<String, String> {
    let mut ctx = BTreeMap::new();
    ctx.insert("title".to_string(), "akusento.ganba.re".to_string());
    ctx.insert("jquery_url".to_string(), JQUERY_URL.to_string());
    ctx
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
    if *APP_INSTALLED.read().expect("Won't fail.") == false { return redirect("/fresh_install", 303) }
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*request).map_err(|e| { internal_error(e); abort(500).unwrap_err() })?;

    let context = new_template_context();

    match user_session {
        Some((_, sess)) => request.app.render_template("main.html", &context)
                            .map(|resp| resp.refresh_cookie(&conn, &sess, request.remote_addr().ip())),
        None => request.app.render_template("hello.html", &context),
    }
}

fn login_form(request: &mut Request) -> PencilResult {
    let app = request.app;
    let ip = request.request.remote_addr.ip();
    let login_form = request.form_mut();
    let email = login_form.take("email").unwrap_or_default();
    let plaintext_pw = login_form.take("password").unwrap_or_default();

    let mut context = new_template_context();
    context.insert("authError".to_string(), "true".to_string());

    do_login(&email, &plaintext_pw, ip)
        .or_else(|e| match e {
            pencil::PencilError::PenHTTPError(pencil::HTTPError::Unauthorized) => {
                warn!("Failed login.");
                let result = app.render_template("hello.html", &context);
                result.map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => Err(e),
        })
}

fn do_login(email : &str, plaintext_pw : &str, ip : IpAddr) -> PencilResult {
    let conn = db_connect().map_err(|_| abort(500).unwrap_err())?;
    let user;
    {
        user = ganbare::auth_user(&conn, email, plaintext_pw, &*RUNTIME_PEPPER)
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
    let conn = db_connect().map_err(|_| abort(500).unwrap_err())?;
    if let Some(session_id) = request.cookies().and_then(get_cookie) {
        ganbare::end_session(&conn, &session_id)
            .map_err(|_| abort(500).unwrap_err())?;
    };

    redirect("/", 303).map(ResponseExt::expire_cookie)
}


fn internal_error<T: std::error::Error>(err: T) -> pencil::PencilError {
    error!("{:?}", err);
    abort(500).unwrap_err()
}


fn confirm_form(request: &mut Request) -> PencilResult {

    let secret = try_or!(request.args().get("secret"), else return abort(400));
    let conn = db_connect()
        .map_err(|e| internal_error(e) )?;
    let (email, _) = ganbare::check_pending_email_confirm(&conn, &secret)
        .map_err(|e| internal_error(e))?;

    let mut context = new_template_context();
    context.insert("email".to_string(), email);
    context.insert("secret".to_string(), secret.clone());

    request.app.render_template("confirm.html", &context)
}

fn confirm_final(req: &mut Request) -> PencilResult {
    req.load_form_data();
    let ip = req.request.remote_addr.ip();
    let conn = db_connect()
        .map_err(|e| internal_error(e))?;
    let secret = try_or!(req.args().get("secret"), else return abort(400)).clone();
    let password = try_or!(req.form().expect("form data loaded.").get("password"), else return abort(400));
    let user = ganbare::complete_pending_email_confirm(&conn, &password, &secret, &*RUNTIME_PEPPER)
        .map_err(|e| match e.kind() {
            &ganbare::errors::ErrorKind::PasswordTooShort => abort(400).unwrap_err(),
            &ganbare::errors::ErrorKind::PasswordTooLong => abort(400).unwrap_err(),
            _ =>  internal_error(e)
        })?;

    do_login(&user.email, &password, ip)
}

macro_rules! parse {
    ($expression:expr) => {$expression.map(String::to_string).ok_or(ErrorKind::FormParseError.to_err())?;}
}

fn add_quiz_form(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let context = new_template_context();

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

    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*req).map_err(|_| abort(500).unwrap_err())?;

    let (user, sess) = try_or!{user_session, else return abort(401)};

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let form = parse_form(&mut *req).map_err(|ee| { error!("{:?}", ee); abort(400).unwrap_err()})?;
    let result = ganbare::create_quiz(&conn, form.0, form.1);
    result.map_err(|e| match e.kind() {
        &ErrorKind::FormParseError => abort(400).unwrap_err(),
        _ => abort(500).unwrap_err(),
    })?;

    redirect("/add_quiz", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

fn add_word_form(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let context = new_template_context();

    req.app.render_template("add_word.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}
fn add_word_post(req: &mut Request) -> PencilResult  {

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

    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let user_session = get_user(&conn, &*req).map_err(|_| abort(500).unwrap_err())?;

    let (user, sess) = try_or!{user_session, else return abort(401)};

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let word = parse_form(req)
            .map_err(|_| abort(400).unwrap_err())?;

    ganbare::create_word(&conn, word)
        .map_err(|_| abort(500).unwrap_err())?;
    
    redirect("/add_word", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}


fn get_line(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let line_id = req.view_args.get("line_id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let line_id = line_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let (file_name, mime_type) = ganbare::get_line_file(&conn, line_id)
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

fn get_image(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

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

fn card_to_json(card: ganbare::Card) -> PencilResult {
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

fn new_quiz(req: &mut Request) -> PencilResult {
    let conn = db_connect()
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
    let conn = db_connect()
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
        .map_err(|e| internal_error(e))?;

    let card = try_or!{new_quiz, else return jsonify(&())}; 
    card_to_json(card).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn change_password_form(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (_, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let mut context = new_template_context();

    let password_changed = req.args_mut().take("password_changed")
        .and_then(|a| if a == "true" { Some(a) } else { None })
        .unwrap_or_else(|| "false".to_string());

    context.insert("password_changed".to_string(), password_changed);

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

    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;

    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    let (old_password, new_password) = parse_form(req)
        .map_err(|_| abort(400).unwrap_err())?;

    match ganbare::auth_user(&conn, &user.email, &old_password, &*RUNTIME_PEPPER) {
        Err(e) => return match e.kind() {
            &ErrorKind::AuthError => {
                let mut context = new_template_context();
                context.insert("authError".to_string(), "true".to_string());

                req.app.render_template("change_password.html", &context)
                    .map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => abort(500),
        },
        Ok(_) => {
            ganbare::change_password(&conn, user.id, &new_password, &*RUNTIME_PEPPER)
                .map_err(|e| match e.kind() {
                    &ErrorKind::PasswordTooShort => abort(400).unwrap_err(),
                    _ => internal_error(e),
                })?;
        },
    };

    redirect("/change_password?password_changed=true", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

fn manage(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;

    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let context = new_template_context();

    req.app.render_template("manage.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


fn get_item(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_word" => {
            let item = ganbare::get_word(&conn, id)
                .map_err(|_| abort(500).unwrap_err())?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
                },
        "get_question" => {
            let item = ganbare::get_question(&conn, id)
                .map_err(|_| abort(500).unwrap_err())?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


fn get_all(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_nuggets" => {
            let items = ganbare::get_skill_nuggets(&conn)
                .map_err(|_| abort(500).unwrap_err())?;
            jsonify(&items)
        },
        "get_bundles" => {
            let items = ganbare::get_audio_bundles(&conn)
                .map_err(|_| abort(500).unwrap_err())?;
            jsonify(&items)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn set_published(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    match endpoint.as_ref() {
        "publish_words" => {
            ganbare::publish_word(&conn, id, true)
                .map_err(|_| abort(500).unwrap_err())?;
        },
        "publish_questions" => {
            ganbare::publish_question(&conn, id, true)
                .map_err(|_| abort(500).unwrap_err())?;
        },
        "unpublish_words" => {
            ganbare::publish_word(&conn, id, false)
                .map_err(|_| abort(500).unwrap_err())?;
        },
        "unpublish_questions" => {
            ganbare::publish_question(&conn, id, false)
                .map_err(|_| abort(500).unwrap_err())?;
        },
        _ => {
            return abort(500)
        },
    };
    let mut resp = Response::new_empty();
    resp.status_code = 204;
    Ok(resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn update_item(req: &mut Request) -> PencilResult {

    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.")
                .parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text)
        .map_err(|_| abort(500).unwrap_err())?;

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
        
            let updated_item = try_or!(ganbare::update_word(&conn, id, item)
                .map_err(|_| abort(500).unwrap_err())?, else return abort(404));

            json = jsonify(&updated_item);

        },
        "update_question" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_question(&conn, id, item)
                .map_err(|_| abort(500).unwrap_err())?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_answer" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_answer(&conn, id, item)
                .map_err(|_| abort(500).unwrap_err())?, else return abort(404));

            json = jsonify(&updated_item);
        },
        _ => return abort(500),
    }
    
    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


fn post_question(req: &mut Request) -> PencilResult {

    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "editors")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text)
        .map_err(|_| abort(500).unwrap_err())?;

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

    let id = ganbare::post_question(&conn, new_qq, new_aas)
            .map_err(|_| abort(500).unwrap_err())?;
        
    let new_url = format!("/api/questions/{}", id);

    redirect(&new_url, 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
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

fn add_users_form(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "admins")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let context = new_template_context();
    req.app.render_template("add_users.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn add_users(req: &mut Request) -> PencilResult {
    let conn = db_connect()
        .map_err(|_| abort(500).unwrap_err())?;
    let (user, sess) = get_user(&conn, req)
        .map_err(|_| abort(500).unwrap_err())?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! ganbare::check_user_group(&conn, &user, "admins")
        .map_err(|_| abort(500).unwrap_err())?
        { return abort(401); }

    let conn = ganbare::db_connect(&*DATABASE_URL).map_err(|e| internal_error(e))?;
    req.load_form_data();
    let form = req.form().expect("The form data is loaded.");
    let emails = try_or!(form.get("emailList"), else return abort(400));
    for row in emails.split("\n") {
        let mut fields = row.split_whitespace();
        let email = try_or!(fields.next(), else return abort(400));
        let mut groups = vec![];
        for field in fields {
            groups.push(try_or!(ganbare::get_group(&conn, field)
                .map_err(|e| internal_error(e))?, else return abort(400)).id);
        }
        let secret = ganbare::add_pending_email_confirm(&conn, email, groups.as_ref())
            .map_err(|e| internal_error(e))?;
        ganbare::email::send_confirmation(email, &secret, &*EMAIL_SERVER, &*EMAIL_DOMAIN, &*SITE_DOMAIN, &**req.app.handlebars_registry.read()
                .expect("The registry is basically read-only after startup."))
            .map_err(|e| internal_error(e))?;
    }

    let context = new_template_context();
    req.app.render_template("add_users.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

fn fresh_install_form(req: &mut Request) -> PencilResult {
    let context = new_template_context();
    req.app.render_template("fresh_install.html", &context)
}

fn fresh_install_post(req: &mut Request) -> PencilResult {
    let form = req.form_mut();
    let email = try_or!(form.take("email"), else return abort(400));
    let new_password = try_or!(form.take("new_password"), else return abort(400));
    let new_password_check = try_or!(form.take("new_password_check"), else return abort(400));
    if new_password != new_password_check { return abort(400) };

    let conn = ganbare::db_connect(&*DATABASE_URL).map_err(|e| internal_error(e))?;
    let user = ganbare::add_user(&conn, &email, &new_password, &*RUNTIME_PEPPER).map_err(|e| internal_error(e))?;
    ganbare::join_user_group_by_name(&conn, &user, "admins").map_err(|e| internal_error(e))?;
    ganbare::join_user_group_by_name(&conn, &user, "editors").map_err(|e| internal_error(e))?;

    { *APP_INSTALLED.write().expect("Can't fail") = true; }

    let mut context = new_template_context();
    context.insert("install_success".into(), "success".into());
    req.app.render_template("fresh_install.html", &context)
}

fn check_env_vars() { &*DATABASE_URL; &*EMAIL_SERVER; &*SITE_DOMAIN; }

fn main() {
    env_logger::init().unwrap();
    info!("Starting.");
    check_env_vars();
    let conn = ganbare::db_connect(&*DATABASE_URL).expect("Can't connect to database!");
    let initialized = ganbare::check_db(&conn).expect("Something funny with the DB!");
    info!("Database OK.");

    { *APP_INSTALLED.write().expect("Can't fail") = initialized; }

    let mut app = Pencil::new(".");
   
    include_templates!(app, "templates", "base.html", "fresh_install.html",
        "hello.html", "main.html", "confirm.html", "add_quiz.html", "add_word.html",
        "manage.html", "change_password.html", "add_users.html", "email_confirm_email.html");
    
    app.enable_static_file_handling();

    app.get("/", "hello", hello);
    app.get("/fresh_install", "fresh_install_form", fresh_install_form);
    app.post("/fresh_install", "fresh_install_post", fresh_install_post);
    app.post("/logout", "logout", logout);
    app.post("/login", "login_form", login_form);
    app.get("/confirm", "confirm_form", confirm_form);
    app.get("/add_quiz", "add_quiz_form", add_quiz_form);
    app.post("/add_quiz", "add_quiz_post", add_quiz_post);
    app.get("/add_users", "add_users_form", add_users_form);
    app.post("/add_users", "add_users", add_users);
    app.get("/add_word", "add_word_form", add_word_form);
    app.post("/add_word", "add_word_post", add_word_post);
    app.get("/manage", "manage", manage);
    app.post("/confirm", "confirm_final", confirm_final);
    app.get("/change_password", "change_password_form", change_password_form);
    app.post("/change_password", "change_password", change_password);
    app.get("/api/nuggets", "get_nuggets", get_all);
    app.get("/api/bundles", "get_bundles", get_all);
    app.get("/api/questions/<id:int>", "get_question", get_item);
    app.get("/api/words/<id:int>", "get_word", get_item);
    app.put("/api/questions/<id:int>?publish", "publish_questions", set_published);
    app.post("/api/question", "post_question", post_question);
    app.put("/api/words/<id:int>?publish", "publish_words", set_published);
    app.put("/api/questions/<id:int>?unpublish", "unpublish_questions", set_published);
    app.put("/api/words/<id:int>?unpublish", "unpublish_words", set_published);
    app.put("/api/words/<id:int>", "update_word", update_item);
    app.put("/api/questions/<id:int>", "update_question", update_item);
    app.put("/api/questions/answers/<id:int>", "update_answer", update_item);
    app.get("/api/new_quiz", "new_quiz", new_quiz);
    app.post("/api/next_quiz", "next_quiz", next_quiz);
    app.get("/api/audio/<line_id:int>", "get_line", get_line);
    app.get("/api/images/<filename:string>", "get_image", get_image);

    info!("Ready. Running on {}, serving at {}", *SERVER_BINDING, *SITE_DOMAIN);
    app.run(*SERVER_BINDING);
}
