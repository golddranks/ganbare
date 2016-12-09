#![feature(field_init_shorthand)]

extern crate ganbare_backend;

#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate mime;
extern crate unicode_normalization;
extern crate tempdir;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]  extern crate lazy_static;

use ganbare_backend::*;
use std::path::PathBuf;
use std::collections::HashSet;

lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    pub static ref AUDIO_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "../audio".into())) };

    pub static ref IMAGE_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_IMAGE_DIR")
        .unwrap_or_else(|_| "../images".into())) };

}

fn clean_audio() {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let fs_files = std::fs::read_dir(&*AUDIO_DIR).unwrap();

    let db_files: HashSet<String> = audio::get_all_files(&conn).unwrap().into_iter().map(|f| f.0).collect();

    let mut trash_dir = AUDIO_DIR.clone();
    trash_dir.push("trash");

    for f in fs_files {
        let f = f.unwrap();
        let f_name = f.file_name();
        if ! db_files.contains(f_name.to_str().unwrap()) && f_name != *"trash" {
            trash_dir.push(&f_name);
            info!("Moving a unneeded file {:?} to the trash directory.", &f_name);
            std::fs::rename(f.path(), &trash_dir).expect("Create \"trash\" directory for cleaning up!");
            trash_dir.pop();
        }
    }
}

fn clean_images() {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    for line in manage::clean_urls(&conn, &*IMAGE_DIR).unwrap() {
        println!("{}", line);
    };
}

fn main() {
    use clap::*;

    env_logger::init().unwrap();
    info!("Starting.");

    App::new("ganba.re audio cleaning tool")
        .version(crate_version!());
    
    clean_audio();
    clean_images();
}
