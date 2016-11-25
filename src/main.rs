#![feature(inclusive_range_syntax)]
#![feature(field_init_shorthand)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;


pub extern crate ganbare_backend;

extern crate pencil;
extern crate dotenv;
extern crate env_logger;
extern crate hyper;
extern crate time;
extern crate rustc_serialize;
extern crate rand;
extern crate chrono;
extern crate regex;
extern crate unicode_normalization;


#[macro_use]
mod helpers;
mod app_pages;
mod manager_pages;
mod http_api;

pub use ganbare_backend as ganbare;
pub use helpers::*;

pub use std::result::Result as StdResult;
pub use pencil::{Request, PencilResult, PencilError};

pub use ganbare::PgConnection;
pub use ganbare::models::{User, Session};
pub use ganbare::errors::ErrorKind::Msg as ErrMsg;
pub use ganbare::errors::Result as Result;
pub use ganbare::errors::{Error, ErrorKind};



use pencil::Pencil;

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
    app.get("/api/audio/<audio_name:string>", "get_audio", http_api::get_audio);
    app.get("/api/images/<filename:string>", "get_image", http_api::get_image);


    info!("Ready. Running on {}, serving at {}", *SERVER_BINDING, *SITE_DOMAIN);
    app.run(*SERVER_BINDING);
}
