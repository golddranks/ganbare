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
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;
extern crate time;

use ganbare_backend::*;
use std::path::PathBuf;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

lazy_static! {

    static ref DATABASE_URL : String = {
        dotenv::dotenv().ok();
        std::env::var("GANBARE_DATABASE_URL")
            .expect(
            "GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)"
            )
    };

    pub static ref AUDIO_DIR : PathBuf = {
        dotenv::dotenv().ok();
        PathBuf::from(std::env::var("GANBARE_AUDIO_DIR")
            .unwrap_or_else(|_| "../audio".into()))
    };

    pub static ref IMAGE_DIR : PathBuf = {
        dotenv::dotenv().ok();
        PathBuf::from(std::env::var("GANBARE_IMAGES_DIR")
            .unwrap_or_else(|_| "../images".into()))
    };
}

pub fn tidy_span_and_br_tags() -> Result<Vec<String>> {
    use ganbare_backend::schema::{words, question_answers};
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let mut logger = vec![];

    let r2 = regex::Regex::new(r#"<span .*?>"#).expect("<- that is a valid regex there");
    let r3 = r#"</span>"#;
    let r4 = regex::Regex::new(r#"<br .*?>"#).expect("<- that is a valid regex there");


    let words: Vec<Word> =
        words::table.filter(words::explanation.like("%span%").or(words::explanation.like("%<br %")))
            .get_results(&conn)?;

    for mut w in words {
        let before = format!("{:?}", w);

        w.explanation = r2.replace_all(&w.explanation, "");
        w.explanation = w.explanation.replace(r3, "");
        w.explanation = r4.replace_all(&w.explanation, "<br>");

        logger.push(format!("Tidied a span/br tag!\n{}\n→\n{:?}\n", before, w));

        let _: Word = w.save_changes(&conn)?;
    }

    let answers: Vec<Answer> =
        question_answers::table.filter(question_answers::answer_text.like("%span%")
                .or(question_answers::answer_text.like("%<br %")))
            .get_results(&conn)?;

    for mut a in answers {
        let before = format!("{:?}", a);

        a.answer_text = r2.replace_all(&a.answer_text, "");
        a.answer_text = a.answer_text.replace(r3, "");
        a.answer_text = r4.replace_all(&a.answer_text, "<br>");

        logger.push(format!("Tidied a span/br tag!\n{}\n→\n{:?}\n", before, a));

        let _: Answer = a.save_changes(&conn)?;
    }

    Ok(logger)
}

pub fn outbound_urls_to_inbound() -> Result<Vec<String>> {
    use ganbare_backend::schema::{words, question_answers};
    use ganbare_backend::manage::sanitize_links;

    let conn = db::connect(&*DATABASE_URL).unwrap();

    let mut logger = vec![];

    let words: Vec<Word> = words::table
        .filter(words::explanation.like("%http://%").or(words::explanation.like("%https://%")))
        .get_results(&conn)?;

    for mut w in words {
        let before = format!("{:?}", w);
        w.explanation = sanitize_links(&w.explanation, &*IMAGE_DIR)?;
        logger.push(format!("Converted an outbound image link to inbound!\n{}\n→\n{:?}\n",
                            before,
                            w));

        let _: Word = w.save_changes(&conn)?;
    }

    let answers: Vec<Answer> =
        question_answers::table.filter(question_answers::answer_text.like("%http://%")
                .or(question_answers::answer_text.like("%https://%")))
            .get_results(&conn)?;

    for mut a in answers {
        let before = format!("{:?}", a);
        a.answer_text = sanitize_links(&a.answer_text, &*IMAGE_DIR)?;
        logger.push(format!("Converted an outbound image link to inbound!\n{}\n→\n{:?}\n",
                            before,
                            a));

        let _: Answer = a.save_changes(&conn)?;
    }

    Ok(logger)
}

fn normalize_unicode() {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let bundles = audio::get_all_bundles(&conn).unwrap();

    for (mut b, _) in bundles {
        let cleaned_name = b.listname.nfc().collect::<String>();
        if cleaned_name != b.listname {
            println!("Non-normalized unicode found: {:?}", b);
            b.listname = cleaned_name;
            let _: AudioBundle = b.save_changes(&conn).unwrap();
        }
    }

    let words: Vec<Word> = schema::words::table.get_results(&conn).unwrap();

    for mut w in words {
        let cleaned_word = w.word.nfc().collect::<String>();
        if cleaned_word != w.word {
            println!("Non-normalized unicode found: {:?}", w);
            w.word = cleaned_word;
            let _: Word = w.save_changes(&conn).unwrap();
        }
    }
    let skills: Vec<SkillNugget> = schema::skill_nuggets::table.get_results(&conn).unwrap();

    for mut s in skills {
        let cleaned_skill = s.skill_summary.nfc().collect::<String>();
        if cleaned_skill != s.skill_summary {
            println!("Non-normalized unicode found: {:?}", s);
            s.skill_summary = cleaned_skill;
            let _: SkillNugget = s.save_changes(&conn).unwrap();
        }
    }
}

fn clean_unused_audio() {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let fs_files = std::fs::read_dir(&*AUDIO_DIR).unwrap();

    let db_files: HashSet<String> =
        audio::get_all_files(&conn).unwrap().into_iter().map(|f| f.0).collect();

    let mut trash_dir = AUDIO_DIR.clone();
    trash_dir.push("trash");

    for f in fs_files {
        let f = f.unwrap();
        let f_name = f.file_name();
        if !db_files.contains(f_name.to_str().unwrap()) && f_name != *"trash" {
            trash_dir.push(&f_name);
            info!("Moving a unneeded file {:?} to the trash directory.",
                  &f_name);
            std::fs::rename(f.path(), &trash_dir)
                .expect("Create \"trash\" directory for cleaning up!");
            trash_dir.pop();
        }
    }

}

use regex::Regex;

lazy_static! {

    static ref IMG_REGEX: Regex = Regex::new(r#"<img[^>]* src="[^"]*/([^"]*)"[^>]*>"#)
        .expect("<- that is a valid regex there");

}

fn clean_unused_images() {
    use ganbare_backend::schema::{question_answers, words};

    let conn = db::connect(&*DATABASE_URL).unwrap();

    let fs_files = std::fs::read_dir(&*IMAGE_DIR).expect(&format!("Not found: {:?}", &*IMAGE_DIR));

    let mut db_files: HashSet<String> = HashSet::new();

    let words: Vec<Word> = words::table.filter(words::explanation.like("%<img%"))
        .get_results(&conn)
        .unwrap();

    for w in words {

        for img_match in IMG_REGEX.captures_iter(&w.explanation) {
            let img = img_match.at(1).expect("The whole match won't match without this submatch.");
            db_files.insert(img.to_string());
        }
    }

    let answers: Vec<Answer> =
        question_answers::table.filter(question_answers::answer_text.like("%<img%"))
            .get_results(&conn)
            .unwrap();

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
        if !db_files.contains(f_name.to_str().unwrap()) && f_name != *"trash" {
            trash_dir.push(&f_name);
            info!("Moving a unneeded file {:?} to the trash directory.",
                  &f_name);
            std::fs::rename(f.path(), &trash_dir)
                .expect("Create \"trash\" directory for cleaning up!");
            trash_dir.pop();
        }
    }
}

lazy_static! {

    static ref BR_IMG_REGEX: Regex = Regex::new(r#"([^>])(<img[^>]* src="[^"]+"[^>]*>)"#)
        .expect("<- that is a valid regex there");

}

fn add_br_between_images_and_text() {
    use ganbare_backend::schema::{question_answers, words};

    let conn = db::connect(&*DATABASE_URL).unwrap();

    let words: Vec<Word> = words::table.filter(words::explanation.like("%<img%"))
        .get_results(&conn)
        .unwrap();

    for mut w in words {
        let new_text = BR_IMG_REGEX.replace_all(&w.explanation, "$1<br>$2");
        if new_text != w.explanation {
            println!("Added a br tag:\n{:?}\n→\n{:?}\n",
                     w.explanation,
                     new_text);
            w.explanation = new_text;
            let _: Word = w.save_changes(&conn).unwrap();
        }
    }

    let answers: Vec<Answer> =
        question_answers::table.filter(question_answers::answer_text.like("%<img%"))
            .get_results(&conn)
            .unwrap();

    for mut a in answers {
        let new_text = BR_IMG_REGEX.replace_all(&a.answer_text, "$1<br>$2");
        if new_text != a.answer_text {
            println!("Added a br tag:\n{:?}\n→\n{:?}\n",
                     a.answer_text,
                     new_text);
            a.answer_text = new_text;
            let _: Answer = a.save_changes(&conn).unwrap();
        }
    }
}

fn main() {
    use clap::*;

    env_logger::init().unwrap();
    info!("Starting.");

    App::new("ganba.re audio cleaning tool").version(crate_version!());


    for line in outbound_urls_to_inbound().unwrap() {
        println!("{}", line);
    }

    for line in tidy_span_and_br_tags().unwrap() {
        println!("{}", line);
    }

    clean_unused_audio();
    clean_unused_images();
    normalize_unicode();
    add_br_between_images_and_text();
}
