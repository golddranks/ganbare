#![feature(field_init_shorthand)]

extern crate ganbare_backend;
extern crate reqwest;
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
use std::path::{Path, PathBuf};
use std::collections::HashSet;

lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    pub static ref AUDIO_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "../audio".into())) };

    pub static ref IMAGE_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_IMAGE_DIR")
        .unwrap_or_else(|_| "../images".into())) };

}

pub fn clean_urls(conn: &PgConnection, image_dir: &Path) -> Result<Vec<String>> {
    use ganbare_backend::schema::{words, question_answers};
    use rand::{thread_rng, Rng};
    use reqwest::header::ContentType;
    use mime::{Mime};
    use mime::TopLevel::{Image};
    use mime::SubLevel::{Png, Jpeg, Gif};

    let mut already_converted = std::collections::HashMap::<String, String>::new();

    let r = regex::Regex::new(r#"['"](https?://.*?(\.[a-zA-Z0-9]{1,4})?)['"]"#).expect("<- that is a valid regex there");

    let mut outbound_to_inbound = |text: &str| -> Result<String> {

        let mut result = text.to_string();

        for url_match in r.captures_iter(text) {

            let url = url_match.at(1).expect("SQL should find stuff that contains this expression");

            if already_converted.contains_key(url) {
                let ref new_url = already_converted[url];
                result = result.replace(url, new_url);

            } else {

                let mut resp = reqwest::get(url).map_err(|_| Error::from("Couldn't load the URL"))?;
        
                let file_extension = url_match.at(2).unwrap_or(".noextension");
        
                let extension = match resp.headers().get::<ContentType>() {
                    Some(&ContentType(Mime(Image, Png, _))) => ".png",
                    Some(&ContentType(Mime(Image, Jpeg, _))) => ".jpg",
                    Some(&ContentType(Mime(Image, Gif, _))) => ".gif",
                    Some(_) => file_extension,
                    None => file_extension,
                };
                
                let mut new_path = image_dir.to_owned();
                let mut filename = "%FT%H-%M-%SZ".to_string();
                filename.extend(thread_rng().gen_ascii_chars().take(10));
                filename.push_str(extension);
                filename = time::strftime(&filename, &time::now()).unwrap();
                new_path.push(&filename);
        
                let mut file = std::fs::File::create(new_path)?;
                std::io::copy(&mut resp, &mut file)?;

                let new_url = String::from("/api/images/")+&filename;
        
                result = result.replace(url, &new_url);
                already_converted.insert(url.to_string(), new_url);
            }

        }
        Ok(result)
    };

    let mut logger = vec![];

    let words: Vec<Word> = words::table
        .filter(words::explanation.like("%http://%").or(words::explanation.like("%https://%")))
        .get_results(conn)?;

    for mut w in words {
        let before = format!("{:?}", w);
        w.explanation = outbound_to_inbound(&w.explanation)?;
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
        a.answer_text = outbound_to_inbound(&a.answer_text)?;
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

fn clean_images() {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    for line in clean_urls(&conn, &*IMAGE_DIR).unwrap() {
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
