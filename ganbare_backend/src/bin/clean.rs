#![feature(field_init_shorthand)]

extern crate ganbare_backend;
extern crate hyper;
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
extern crate rand;
extern crate regex;
extern crate time;

use ganbare_backend::*;
use std::path::{PathBuf};
use std::collections::HashSet;

lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    pub static ref AUDIO_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "../audio".into())) };

    pub static ref IMAGE_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_IMAGE_DIR")
        .unwrap_or_else(|_| "../images".into())) };

}

pub fn clean_urls(conn: &PgConnection) -> Result<Vec<String>> {
    use ganbare_backend::schema::{words, question_answers};
    use ganbare_backend::manage::sanitize_links;


    let mut logger = vec![];

    let words: Vec<Word> = words::table
        .filter(words::explanation.like("%http://%").or(words::explanation.like("%https://%")))
        .get_results(conn)?;

    for mut w in words {
        let before = format!("{:?}", w);
        w.explanation =  sanitize_links(&w.explanation, &*IMAGE_DIR)?;
        logger.push(format!("Converted an outbound image link to inbound!\n{}\n→\n{:?}\n", before, w));

        let _ : Word = w.save_changes(conn)?;
    }

    let words: Vec<Word> = words::table
        .filter(words::explanation.like("%span%"))
        .get_results(conn)?;

    let r2 = regex::Regex::new(r#"<span .*?>"#).expect("<- that is a valid regex there");
    let r3 = regex::Regex::new(r#"</span>"#).expect("<- that is a valid regex there");

    for mut w in words {
        let before = format!("{:?}", w);

        w.explanation = r2.replace_all(&w.explanation, "");
        w.explanation = r3.replace_all(&w.explanation, "");

        logger.push(format!("Removed a span!\n{}\n→\n{:?}\n", before, w));

        let _ : Word = w.save_changes(conn)?;
    }

    let answers: Vec<Answer> = question_answers::table
        .filter(question_answers::answer_text.like("%http://%").or(question_answers::answer_text.like("%https://%")))
        .get_results(conn)?;

    for mut a in answers {
        let before = format!("{:?}", a);
        a.answer_text =  sanitize_links(&a.answer_text, &*IMAGE_DIR)?;
        logger.push(format!("Converted an outbound image link to inbound!\n{}\n→\n{:?}\n", before, a));

        let _ : Answer = a.save_changes(conn)?;
    }

    let answers: Vec<Answer> = question_answers::table
        .filter(question_answers::answer_text.like("%span%"))
        .get_results(conn)?;

    let r2 = regex::Regex::new(r#"<span .*?>"#).expect("<- that is a valid regex there");
    let r3 = regex::Regex::new(r#"</span>"#).expect("<- that is a valid regex there");

    for mut a in answers {
        let before = format!("{:?}", a);

        a.answer_text = r2.replace_all(&a.answer_text, "");
        a.answer_text = r3.replace_all(&a.answer_text, "");

        logger.push(format!("Removed a span!\n{}\n→\n{:?}\n", before, a));

        let _ : Answer = a.save_changes(conn)?;
    }

    Ok(logger)
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

use regex::Regex;

lazy_static! {

    static ref IMG_REGEX: Regex = Regex::new(r#"<img[^>]* src="[^"]*/([^"]*)"[^>]*>"#)
        .expect("<- that is a valid regex there");

}

fn clean_images() {
    use ganbare_backend::schema::{question_answers, words};

    let conn = db::connect(&*DATABASE_URL).unwrap();

    let fs_files = std::fs::read_dir(&*IMAGE_DIR).expect(&format!("Not found: {:?}", &*IMAGE_DIR));

    let mut db_files: HashSet<String> = HashSet::new();

    for line in clean_urls(&conn).unwrap() {
        println!("{}", line);
    };

    let words: Vec<Word> = words::table
        .filter(words::explanation.like("%<img%"))
        .get_results(&conn).unwrap();

    for w in words {

        for img_match in IMG_REGEX.captures_iter(&w.explanation) {
            let img = img_match.at(1).expect("The whole match won't match without this submatch.");
            db_files.insert(img.to_string());
        }
    }

    let answers: Vec<Answer> = question_answers::table
        .filter(question_answers::answer_text.like("%<img%"))
        .get_results(&conn).unwrap();

    for a in answers {
        for img_match in IMG_REGEX.captures_iter(&a.answer_text) {
            let img = img_match.at(1).expect("The whole match won't match without this submatch.");
            db_files.insert(img.to_string());
        }
    }

    let mut trash_dir = IMAGE_DIR.clone();
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

fn main() {
    use clap::*;

    env_logger::init().unwrap();
    info!("Starting.");

    App::new("ganba.re audio cleaning tool")
        .version(crate_version!());
    
    clean_audio();
    clean_images();
}
