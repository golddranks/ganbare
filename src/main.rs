mod helpers;
mod app_pages;
mod manager_pages;
mod http_api;
mod test;

pub use ganbare::DEV_MODE;
pub use ganbare_backend as ganbare;
pub use helpers::*;

use lazy_static::lazy_static;
use log::{info, debug, error, warn};
pub use std::result::Result as StdResult;
pub use pencil::{Request, PencilResult};

pub use ganbare::models::{User, Session};
pub use ganbare::errors::ErrorKind::Msg as ErrMsg;
pub use ganbare::errors::Result;
pub use ganbare::errors::{Error, ErrorKind};
pub use ganbare::Connection;
use lettre::transport::EmailTransport;
pub use ganbare::helpers::Cache;
use std::time::Instant;


pub fn favicon(_: &mut Request) -> PencilResult {
    use pencil::helpers::send_file;
    send_file("static/images/speaker_pink.png",
              "image/x-icon".parse().expect("We now statically this mime is good"),
              false)
            .set_static_cache()
}

pub fn health_check(_: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    ganbare_backend::user::get_group(&conn, "admins").err_500()?;
    Ok(Response::from("OK."))
}

pub fn source_maps(req: &mut Request) -> PencilResult {
    if !*ENABLE_SOURCE_MAPS {
        return pencil::helpers::abort(404);
    }
    use pencil::send_from_directory;
    let file_path = req.view_args
        .get("file_path")
        .expect("Pencil guarantees that filename should exist as an arg.");
    send_from_directory("src", file_path, false)
}

use std::collections::VecDeque;
use std::sync::RwLock;
use lettre::email::Email;
use lettre::transport::smtp::SmtpTransportBuilder;

lazy_static! {
    pub static ref AUDIO_CACHE: Cache<u64, Vec<u8>> =
        Cache::new(Duration::from_secs(60*2));
    pub static ref LOGGED_OUT_CACHE: Cache<i32, ganbare::session::UserSession> =
        Cache::new(Duration::from_secs(60*10));
    pub static ref MAIL_QUEUE: RwLock<VecDeque<Email>> =
        RwLock::new(VecDeque::new());
}

#[allow(unused_mut)]
pub fn background_control_thread() {
    use std::thread::sleep;
    use std::error::Error;

    let conn;
    loop {
        match db_connect() {
            Ok(c) => {
                conn = c;
                break;
            }
            Err(e) => error!("background_control_thread::db_connect: Error: {}", e),
        };
        sleep(Duration::from_secs(5));
    }

    let mut app = Pencil::new(".");
    include_templates!(app, "templates", "slacker_heatenings.html");

    let mut mailer = SmtpTransportBuilder::new(&*EMAIL_SERVER)
        .expect("Couldn't setup the email transport!")
        .encrypt()
        .credentials(EMAIL_SMTP_USERNAME.as_str(), EMAIL_SMTP_PASSWORD.as_str())
        .build();

    loop {
        sleep(Duration::from_secs(5));

        match ganbare::email::send_nag_emails(&*MAIL_QUEUE,
                                              &conn,
                                              *NAG_EMAIL_ABSENCE_PERIOD,
                                              *NAG_EMAIL_GRACE_PERIOD,
                                              &*SITE_DOMAIN,
                                              &*SITE_LINK,
                                              &*app.handlebars_registry
                                                  .read()
                                                  .expect("The registry is basically \
                                                           read-only after startup."),
                                              (&*EMAIL_ADDRESS, &*EMAIL_NAME)) {
            Ok(()) => (),
            Err(e) => {
                error!("background_control_thread::send_nag_emails: Error: {}. Cause: {:?}",
                       e,
                       e.source());
            }
        };

        match ganbare::session::clean_old_sessions(&conn, *SESSION_EXPIRE) {
            Ok(count) => {
                if count != 0 {
                    info!("Deleted {} expired sessions.", count)
                }
            }
            Err(e) => {
                error!("background_control_thread::clean_old_sessions: Error: {}",
                       e)
            }
        };

        match ganbare::email::clean_old_pendings(&conn, *EMAIL_EXPIRE) {
            Ok(count) => {
                if count != 0 {
                    info!("Deleted {} unanswered email confirmations.", count);
                }
            }
            Err(e) => {
                error!("background_control_thread::clean_old_pendings: Error: {}",
                       e)
            }
        }

        match AUDIO_CACHE.clean_expired() {
            Ok((remaining, removed)) if removed > 0 => {
                debug!("Removed an old temp audio recordings. Remaining: {}, Removed: {}",
                       remaining,
                       removed);
            }
            Err(e) => {
                error!("background_control_thread::AUDIO_CACHE.clean_expired: Error: {}",
                       e)
            }
            _ => (),
        }

        match LOGGED_OUT_CACHE.clean_expired() {
            Ok((remaining, removed)) if removed > 0 => {
                debug!("Removed logged out sessions from memory cache. Remaining: {}, Removed: {}",
                       remaining,
                       removed);
            }
            Err(e) => {
                error!("background_control_thread::LOGGED_OUT_CACHE.clean_expired: Error: {}",
                       e)
            }
            _ => (),
        }

        let mut outgoing_mails = vec![];
        if let Ok(mut mails) =
            MAIL_QUEUE.try_write().or_else(|e| {
                                               debug!("The queue is locked. Skipping.");
                                               Err(e)
                                           }) {
            outgoing_mails.extend(mails.drain(..));
        }
        if ! outgoing_mails.is_empty() {
            info!("Sending {} mails!", outgoing_mails.len());
            for email in outgoing_mails.drain(..) {
                match mailer.send(email) {
                    Ok(_) => (),
                    Err(e) => error!("Couldn't send email! Error: {}", e),
                }
            }
            debug!("Mails sent!");
        }
    }
}

fn csrf_check(req: &mut Request) -> Option<PencilResult> {
    use hyper::header::{Origin, Referer, Host};
    use url::Url;

    let origin = req.headers().get();
    let referer = req.headers().get();
    let method_mutating = !req.method().safe();
    let url = req.url.path();

    if method_mutating || (*PARANOID && url.starts_with("/api")) {
        // Enable anti-CSRF heuristics:
        // when the method is POST, DELETE etc., or if the request uses the HTTP API.

        if req.host.hostname != &**SITE_DOMAIN {
            return Some(Ok(bad_request(format!("The host field is wrong. Expected: {}, Got: {}",
                                            &**SITE_DOMAIN,
                                            req.host.hostname))));
        }

        if let Some(&Origin { host: Host { ref hostname, .. }, .. }) = origin {
            if hostname != &**SITE_DOMAIN {
                println!("Someone tried to do a request with a wrong Origin: {} Possible CSRF? \
                        Details: {:?}, {:?}",
                         hostname,
                         origin,
                         referer);
                return Some(pencil::abort(403));
            }
        }
        if let Some(&Referer(ref referer)) = referer {
            let url = Url::parse(referer);
            let hostname = match url.as_ref().map(|url| url.host_str()) {
                Ok(Some(host)) => host,
                Ok(None) | Err(_) => return Some(pencil::abort(400)),
            };
            if hostname != &**SITE_DOMAIN {
                println!("Someone tried to do a request with a wrong Referer: {} Possible CSRF?",
                         hostname);
                return Some(pencil::abort(403));
            }
        }
        if origin.is_none() && referer.is_none() {
            println!("Someone tried to do a request with no Referer or Origin while triggering \
                    the anti-CSRF heuristics!");
            println!("Accessing with HTTP method: {:?}. The first segment of path: {:?}",
                     req.method(),
                     url);
            return Some(pencil::abort(403));
        }
    }

    None
}

fn set_headers(_req: &Request, resp: &mut pencil::Response) {
    use hyper::*;
    use hyper::header::*;

    header! {
        (ContentSecurityPolicy, "Content-Security-Policy") => [String]
    }

    if *PARANOID {
        resp.headers.set(ContentSecurityPolicy(CONTENT_SECURITY_POLICY.clone()));
        resp.headers.set(StrictTransportSecurity {
                             include_subdomains: true,
                             max_age: 31536000,
                         });
    }
}

#[allow(unused_variables)]
fn resp_time_start(req: &mut Request) -> Option<PencilResult> {
    if *PERF_TRACE {
        let start = Instant::now();
        req.extensions_data.insert::<RequestTime>(start);
    }
    None
}


#[allow(unused_variables)]
fn resp_time_stop(req: &Request, resp: &mut pencil::Response) {
    let start = req.extensions_data.get::<RequestTime>().unwrap();
    let lag = start.elapsed();
    debug!("Request from {} to {} {}, response: {} took {} ms", req.remote_addr, req.method, req.url, resp.status_code, start.elapsed().as_millis());
}

use pencil::{Pencil, Response};

#[allow(dead_code)]
struct RequestTime;

impl typemap::Key for RequestTime {
    type Value = Instant;
}

pub fn main() {
    std::env::set_var("RUST_LOG", helpers::LOG.to_owned());
    pretty_env_logger::init();
    info!("Starting in {} mode.", if DEV_MODE { "dev" } else { "release" });
    check_env_vars();
    debug!("Env vars OK.");
    let conn = db_connect().expect("Can't connect to database!");
    debug!("Connected to DB.");
    let installed = ganbare::db::check(&conn).expect("Something funny with the DB!");
    info!("Database OK. Installed: {}", installed);

    debug!("Initing Pencil.");
    let mut app = Pencil::new(".");

    include_templates!(app,
                       "templates",
                       "base.html",
                       "fresh_install.html",
                       "welcome.html",
                       "reset_password.html",
                       "send_mail.html",
                       "retelling.html",
                       "hello.html",
                       "main.html",
                       "confirm.html",
                       "add_quiz.html",
                       "add_word.html",
                       "survey.html",
                       "audio.html",
                       "send_pw_reset_email.html",
                       "events.html",
                       "manage.html",
                       "change_password.html",
                       "add_users.html",
                       "email_confirm_email.html",
                       "pw_reset_email.html",
                       "users.html",
                       "slacker_heatenings.html",
                       "agreement.html",
                       "info.html",
                       "pretest_info.html",
                       "pretest_done.html",
                       "posttest_info.html",
                       "posttest_done.html");

    app.enable_static_cached_file_handling(Duration::from_secs(*CACHE_MAX_AGE as u64));

    // Note: resp_time_start MUST be the first one
    if *PERF_TRACE { app.before_request(resp_time_start); }
    app.before_request(|req| { info!("Request from {} to {} {}", req.remote_addr, req.method, req.url); None });
    app.before_request(check_if_cached);
    app.before_request(csrf_check);
    app.after_request(set_headers);
    app.teardown_request(|e| { e.map(|e| error!("Error: {}", e));});
    if *PERF_TRACE { app.after_request(resp_time_stop); }

    // DEBUGGING
    app.get("/src/<file_path:path>", "source_maps", source_maps);

    // BASIC FUNCTIONALITY
    app.get("/health_check", "health_check", health_check);
    app.get("/favicon.ico", "favicon", favicon);
    app.get("/", "hello", app_pages::hello);
    app.get("/welcome", "welcome", app_pages::text_pages);
    app.get("/agreement", "agreement", app_pages::text_pages);
    app.get("/info", "info", app_pages::text_pages);
    app.get("/survey", "survey", app_pages::survey);
    app.get("/pretest_info", "pretest_info", app_pages::text_pages);
    app.get("/pretest", "pretest", app_pages::pre_post_test);
    app.get("/pretest_retelling",
            "pretest_retelling",
            app_pages::retelling);
    app.get("/pretest_done", "pretest_done", app_pages::text_pages);
    app.get("/sorting", "sorting", app_pages::sorting_ceremony);
    app.get("/posttest_info", "posttest_info", app_pages::text_pages);
    app.get("/posttest", "posttest", app_pages::pre_post_test);
    app.get("/posttest_retelling",
            "posttest_retelling",
            app_pages::retelling);
    app.get("/posttest_done", "posttest_done", app_pages::text_pages);
    app.post("/ok", "ok", app_pages::ok);
    app.get("/login", "login_form", app_pages::login_form);
    app.post("/login", "login_post", app_pages::login_post);
    app.post("/logout", "logout", app_pages::logout);
    app.get("/logout?doit", "logout_get", app_pages::logout);
    app.get("/confirm", "confirm_form", app_pages::confirm_form);
    app.post("/confirm", "confirm_post", app_pages::confirm_post);
    app.get("/change_password",
            "change_password_form",
            app_pages::change_password_form);
    app.post("/change_password",
             "change_password",
             app_pages::change_password);
    app.get("/reset_password?secret=<secret:string>",
            "reset_password_form",
            app_pages::confirm_password_reset_form);
    app.get("/reset_password?changed=true",
            "password_reset_success",
            app_pages::password_reset_success);
    app.post("/reset_password",
             "reset_password_post",
             app_pages::confirm_password_reset_post);
    app.get("/send_password_reset_email",
            "pw_reset_email_form",
            app_pages::pw_reset_email_form);
    app.post("/send_password_reset_email",
             "send_pw_reset_email",
             app_pages::send_pw_reset_email);

    // MANAGER PAGES
    app.get("/fresh_install",
            "fresh_install_form",
            manager_pages::fresh_install_form);
    app.post("/fresh_install",
             "fresh_install_post",
             manager_pages::fresh_install_post);
    app.get("/add_quiz", "add_quiz_form", manager_pages::add_quiz_form);
    app.post("/add_quiz", "add_quiz_post", manager_pages::add_quiz_post);
    app.get("/add_users",
            "add_users_form",
            manager_pages::add_users_form);
    app.post("/add_users", "add_users", manager_pages::add_users);
    app.get("/add_word", "add_word_form", manager_pages::add_word_form);
    app.post("/add_word", "add_word_post", manager_pages::add_word_post);
    app.get("/manage", "manage", manager_pages::manage);
    app.get("/users", "users", manager_pages::users);
    app.get("/events", "events", manager_pages::events);
    app.get("/audio", "audio", manager_pages::audio);
    app.get("/send_mail",
            "send_mail_form",
            manager_pages::send_mail_form);
    app.post("/send_mail",
             "send_mail_post",
             manager_pages::send_mail_post);

    // HTTP API
    app.post("/api/mic_check?<random_token:string>",
             "mic_check_rec",
             http_api::mic_check);
    app.get("/api/mic_check.ogg?<random_token:string>",
            "mic_check_play",
            http_api::mic_check);
    app.get("/api/next_retelling?event=<event_name:string>",
            "next_retelling",
            http_api::next_retelling);
    app.get("/api/new_retelling?event=<event_name:string>",
            "new_retelling",
            http_api::new_retelling);
    app.get("/api/build_number",
            "get_build_number",
            http_api::get_build_number);
    app.get("/api/user_audio.ogg?event=<event_name:string>&quiz_number=<quiz_number:\
             int>&rec_number=<rec_number:int>",
            "get_useraudio",
            http_api::get_useraudio);
    app.post("/api/user_audio?event=<event_name:string>",
             "post_useraudio",
             http_api::post_useraudio);
    app.get("/api/nuggets", "get_nuggets", http_api::get_all);
    app.get("/api/users", "get_users", http_api::get_all);
    app.get("/api/users/<id:int>/skills",
            "get_skills",
            http_api::get_user_details);
    app.get("/api/users/<id:int>/asked_items",
            "get_asked_items",
            http_api::get_user_details);
    app.get("/api/events", "get_events", http_api::get_all);
    app.put("/api/events/<id:int>",
            "update_event",
            http_api::update_item);
    app.get("/api/bundles", "get_bundles", http_api::get_all);
    app.delete("/api/bundles/<id_from:int>?merge_with=<id_to:int>",
               "merge_bundle",
               http_api::merge_item);
    app.delete("/api/bundles/<id:int>", "del_bundle", http_api::del_item);
    app.put("/api/bundles/<id:int>",
            "update_bundle",
            http_api::update_item);
    app.put("/api/audio_files/<id:int>",
            "update_audio_file",
            http_api::update_item);
    app.get("/api/narrators", "get_narrators", http_api::get_all);
    app.delete("/api/narrators/<id_from:int>?merge_with=<id_to:int>",
               "merge_narrator",
               http_api::merge_item);
    app.delete("/api/narrators/<id:int>",
               "del_narrator",
               http_api::del_item);
    app.put("/api/narrators/<id:int>",
            "update_narrator",
            http_api::update_item);
    app.get("/api/questions/<id:int>",
            "get_question",
            http_api::get_item);
    app.get("/api/exercises/<id:int>",
            "get_exercise",
            http_api::get_item);
    app.get("/api/words/<id:int>", "get_word", http_api::get_item);
    app.post("/api/questions", "post_question", http_api::post_question);
    app.post("/api/exercises", "post_exercise", http_api::post_exercise);
    app.delete("/api/words/<id:int>", "del_word", http_api::del_item);
    app.delete("/api/questions/<id:int>",
               "del_question",
               http_api::del_item);
    app.delete("/api/exercises/<id:int>",
               "del_exercise",
               http_api::del_item);
    app.put("/api/users/<user_id:int>?add_group=<group_id:int>",
            "add_group",
            http_api::user);
    app.put("/api/users/<user_id:int>?remove_group=<group_id:int>",
            "remove_group",
            http_api::user);
    app.put("/api/users/<user_id:int>?settings=metrics",
            "set_metrics",
            http_api::user);
    app.get("/api/groups", "get_groups", http_api::get_all);
    app.delete("/api/users/<id:int>", "del_user", http_api::del_item);
    app.delete("/api/users/<id:int>/due_and_pending_items",
               "del_due_and_pending_items",
               http_api::del_item);
    app.delete("/api/skills/<id:int>", "del_skill", http_api::del_item);
    app.delete("/api/events/<id:int>/<user_id:int>",
               "del_event_exp",
               http_api::del_item);
    app.put("/api/questions/<id:int>?publish",
            "publish_questions",
            http_api::set_published);
    app.put("/api/questions/<id:int>?unpublish",
            "unpublish_questions",
            http_api::set_published);
    app.put("/api/words/<id:int>?publish",
            "publish_words",
            http_api::set_published);
    app.put("/api/words/<id:int>?unpublish",
            "unpublish_words",
            http_api::set_published);
    app.put("/api/exercises/<id:int>?publish",
            "publish_exercises",
            http_api::set_published);
    app.put("/api/exercises/<id:int>?unpublish",
            "unpublish_exercises",
            http_api::set_published);
    app.put("/api/words/<id:int>", "update_word", http_api::update_item);
    app.put("/api/questions/<id:int>",
            "update_question",
            http_api::update_item);
    app.put("/api/questions/answers/<id:int>",
            "update_answer",
            http_api::update_item);
    app.put("/api/exercises/<id:int>",
            "update_exercise",
            http_api::update_item);
    app.put("/api/exercises/variants/<id:int>",
            "update_variant",
            http_api::update_item);

    app.get("/api/new_quiz", "new_quiz", http_api::new_quiz);
    app.post("/api/next_quiz", "next_quiz", http_api::next_quiz);

    app.get("/api/new_quiz_testing",
            "new_quiz_testing",
            http_api::new_quiz_testing);
    app.post("/api/next_quiz_testing",
             "next_quiz_testing",
             http_api::next_quiz_testing);

    app.get("/api/audio/<audio_name:string>",
            "get_audio",
            http_api::get_audio);
    app.get("/api/audio.mp3?<audio_name:string>",
            "quiz_audio",
            http_api::quiz_audio);
    app.get("/api/images/<filename:string>",
            "get_image",
            http_api::get_image);
    app.put("/api/eventdata/<eventname:string>/<key:string>",
            "put_eventdata",
            http_api::save_eventdata);
    app.post("/api/eventdata/<eventname:string>",
             "post_eventdata",
             http_api::save_eventdata);

    std::thread::spawn(background_control_thread);

    info!("Ready. Running on http://{}, serving at {} with {} threads",
          *SERVER_BINDING,
          *SITE_DOMAIN,
          *SERVER_THREADS);
    app.run_threads(*SERVER_BINDING, *SERVER_THREADS);
}
