#![feature(inclusive_range_syntax)]
#![feature(proc_macro)]
#![feature(field_init_shorthand)]
#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen)]

#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;


pub extern crate ganbare_backend;

extern crate try_map;
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
extern crate mime;

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


pub fn favicon(_: &mut Request) -> PencilResult {
    use pencil::helpers::send_file;
    send_file("static/images/speaker_pink.png", "image/x-icon".parse().unwrap(), false, None)
}


use pencil::Pencil;

pub fn main() {
    env_logger::init().unwrap();
    info!("Starting.");
    check_env_vars();
    let conn = ganbare::db::connect(&*DATABASE_URL).expect("Can't connect to database!");
    ganbare::db::check(&conn).expect("Something funny with the DB!");
    info!("Database OK.");

    let mut app = Pencil::new(".");
   
    include_templates!(app, "templates", "base.html", "fresh_install.html", "welcome.html", "join.html",
        "hello.html", "main.html", "confirm.html", "add_quiz.html", "add_word.html", "survey.html", "audio.html",
        "manage.html", "change_password.html", "add_users.html", "email_confirm_email.html", "users.html");
    
    app.enable_static_file_handling();

    // BASIC FUNCTIONALITY
    app.get("/favicon.ico", "favicon", favicon);
    app.get("/", "hello", app_pages::hello);
    app.get("/welcome", "welcome", app_pages::welcome);
    app.get("/survey", "survey", app_pages::survey);
    app.post("/ok", "ok", app_pages::ok);
    app.get("/login", "login_form", app_pages::login_form);
    app.post("/login", "login_post", app_pages::login_post);
    app.post("/logout", "logout", app_pages::logout);
    app.get("/join", "join", app_pages::join_form);
    app.post("/join", "join", app_pages::join_post);
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
    app.get("/users", "users", manager_pages::users);
    app.get("/audio", "audio", manager_pages::audio);

    // HTTP API
    app.get("/api/nuggets", "get_nuggets", http_api::get_all);
    app.get("/api/users", "get_users", http_api::get_all);
    app.get("/api/bundles", "get_bundles", http_api::get_all);
    app.delete("/api/bundles/<id_from:int>?merge_with=<id_to:int>", "merge_bundle", http_api::merge_item);
    app.delete("/api/bundles/<id:int>", "del_bundle", http_api::del_item);
    app.put("/api/bundles/<id:int>", "update_bundle", http_api::update_item);
    app.get("/api/narrators", "get_narrators", http_api::get_all);
    app.delete("/api/narrators/<id_from:int>?merge_with=<id_to:int>", "merge_narrator", http_api::merge_item);
    app.delete("/api/narrators/<id:int>", "del_narrator", http_api::del_item);
    app.put("/api/narrators/<id:int>", "update_narrator", http_api::update_item);
    app.get("/api/questions/<id:int>", "get_question", http_api::get_item);
    app.get("/api/exercises/<id:int>", "get_exercise", http_api::get_item);
    app.get("/api/words/<id:int>", "get_word", http_api::get_item);
    app.post("/api/questions", "post_question", http_api::post_question);
    app.post("/api/exercises", "post_exercise", http_api::post_exercise);
    app.delete("/api/words/<id:int>", "del_word", http_api::del_item);
    app.delete("/api/questions/<id:int>", "del_question", http_api::del_item);
    app.delete("/api/exercises/<id:int>", "del_exercise", http_api::del_item);
    app.put("/api/users/<user_id:int>?add_group=<group_id:int>", "add_group", http_api::user);
    app.put("/api/users/<user_id:int>?remove_group=<group_id:int>", "remove_group", http_api::user);
    app.put("/api/users/<user_id:int>?settings=metrics", "set_metrics", http_api::user);
    app.delete("/api/users/<id:int>", "del_user", http_api::del_item);
    app.delete("/api/skills/<id:int>", "del_skill", http_api::del_item);
    app.put("/api/questions/<id:int>?publish", "publish_questions", http_api::set_published);
    app.put("/api/questions/<id:int>?unpublish", "unpublish_questions", http_api::set_published);
    app.put("/api/words/<id:int>?publish", "publish_words", http_api::set_published);
    app.put("/api/words/<id:int>?unpublish", "unpublish_words", http_api::set_published);
    app.put("/api/exercises/<id:int>?publish", "publish_exercises", http_api::set_published);
    app.put("/api/exercises/<id:int>?unpublish", "unpublish_exercises", http_api::set_published);
    app.put("/api/words/<id:int>", "update_word", http_api::update_item);
    app.put("/api/questions/<id:int>", "update_question", http_api::update_item);
    app.put("/api/questions/answers/<id:int>", "update_answer", http_api::update_item);
    app.get("/api/new_quiz", "new_quiz", http_api::new_quiz);
    app.post("/api/next_quiz", "next_quiz", http_api::next_quiz);
    app.get("/api/audio/<audio_name:string>", "get_audio", http_api::get_audio);
    app.get("/api/audio.mp3?<audio_name:string>", "quiz_audio", http_api::quiz_audio);
    app.get("/api/images/<filename:string>", "get_image", http_api::get_image);
    app.put("/api/eventdata/<eventname:string>/<key:string>", "put_eventdata", http_api::save_eventdata);
    app.post("/api/eventdata/<eventname:string>", "post_eventdata", http_api::save_eventdata);


    info!("Ready. Running on {}, serving at {}", *SERVER_BINDING, *SITE_DOMAIN);
    app.run(*SERVER_BINDING);
}
