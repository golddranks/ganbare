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

use unicode_normalization::UnicodeNormalization;
use ganbare_backend::*;
use std::path::PathBuf;

lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

    pub static ref AUDIO_DIR : PathBuf = { dotenv::dotenv().ok(); PathBuf::from(std::env::var("GANBARE_AUDIO_DIR")
        .unwrap_or_else(|_| "../audio".into())) };

}

fn import_batch(path: &str, narrator: &str, sentences: bool) {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let files = std::fs::read_dir(path).unwrap();

    let tmp_dir = tempdir::TempDir::new("ganbare_backend_import_tool").expect("create temp dir");
    for f in files {
        let f = f.unwrap();
        let path = f.path();
        let extension = if let Some(Some(e)) = path.extension().map(|s|s.to_str()) { e } else { continue }.to_owned();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

        let mut word = path.file_stem().unwrap().to_str().unwrap().nfc().collect::<String>();
        let last_char = word.chars().next_back().expect("The word surely is longer than 0 characters!");

        if !sentences && word.chars().filter(|c| c == &'・' || c == &'*').count() > 1 {
            panic!("Invalid filename! More than one accent marks: {:?}", word);
        }
        if sentences && !word.contains('、') {
            panic!("Invalid filename! No 、: {:?}", word);
        }

        if last_char.is_digit(10) {
            word.pop();
        }

        let temp_file_path = tmp_dir.path().join(f.file_name());

        std::fs::copy(path, &temp_file_path).expect("copying files");

        if extension != "mp3" { continue };

        use std::str::FromStr;
        let mime = mime::Mime::from_str("audio/mpeg").unwrap();

        let files = vec![(temp_file_path, Some(file_name), mime)];

        let w = if sentences {
            full_sentence(&conn, &word, narrator, files)
        } else {
            simple_word(&word, narrator, files)
        };

        println!("{:?}", w);

        manage::create_or_update_word(&conn, w, &AUDIO_DIR).unwrap();
    }
}

fn full_sentence<'a>(conn: &PgConnection, filename: &str, narrator: &'a str, files: Vec<(PathBuf, Option<String>, mime::Mime)>) -> manage::NewWordFromStrings<'a> {

    let mut word_split = filename.split('、');
    let mut word = word_split.next().unwrap().to_owned();
    let sentence = word_split.next().unwrap().to_owned();

    let nugget = word.replace("・", "");

    println!("{:?}", &word);
    use schema::words;
    let explanation: String = words::table
        .filter(words::word.eq(&word))
        .select(words::explanation)
        .get_result(conn).unwrap();

    if !word.contains('・') {
        word.push('＝');
    }

    let second_codepoint = word.char_indices().nth(1).unwrap().0;
    if word.chars().nth(1).unwrap() == '・' {
        word.insert(0, '／');
    } else {
        word.insert(second_codepoint, '／');
    }

    let sentence = sentence.replace(&nugget, &word);

    manage::NewWordFromStrings {
        word: sentence,
        explanation,
        nugget,
        narrator,
        files,
        skill_level: 5,
        priority: 0,
    }
}

fn simple_word<'a>(filename: &str, narrator: &'a str, files: Vec<(PathBuf, Option<String>, mime::Mime)>) -> manage::NewWordFromStrings<'a> {

    let nugget = filename.replace("*", "").replace("・", "");
    let word = filename.to_owned();

    manage::NewWordFromStrings {
        word,
        explanation: "".into(),
        nugget,
        narrator,
        files,
        skill_level: 0,
        priority: 0,
    }
}

fn main() {
    use clap::*;

    env_logger::init().unwrap();
    info!("Starting.");

    let matches = App::new("ganba.re word import tool")
        .arg(Arg::with_name("sentences")
                                .short("s")
                                .long("sentences")
                                .help("Flag to enable importing sentences"))
        .arg(Arg::with_name("PATH")
                                .index(1)
                                .required(true)
                                .value_name("PATH")
                                .help("The path to the input files")
                                .takes_value(true))
        .arg(Arg::with_name("NARRATOR")
                                .index(2)
                                .required(true)
                                .value_name("NARRATOR")
                                .help("The person narrating the files")
                                .takes_value(true))
        .version(crate_version!())
        .get_matches();
    
    let sentences = matches.is_present("sentences");
    let path = matches.value_of("PATH").unwrap();
    let narrator = matches.value_of("NARRATOR").unwrap();

    import_batch(&path, &narrator, sentences);
}
