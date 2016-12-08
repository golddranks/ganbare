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

fn import_batch(path: &str, narrator: &str) {
    let conn = db::connect(&*DATABASE_URL).unwrap();

    let files = std::fs::read_dir(path).unwrap();

    let tmp_dir = tempdir::TempDir::new("ganbare_backend_import_tool").expect("create temp dir");
    for f in files {
        let f = f.unwrap();
        let path = f.path();
        let extension = if let Some(Some(e)) = path.extension().map(|s|s.to_str()) { e } else { continue }.to_string();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        let mut word = path.file_stem().unwrap().to_str().unwrap().nfc().collect::<String>();
        let last_char = word.chars().next_back().expect("The word surely is longer than 0 characters!");

        if last_char.is_digit(10) {
            word.pop();
        }

        let temp_file_path = tmp_dir.path().join(f.file_name());

        std::fs::copy(path, &temp_file_path).expect("copying files");

        if extension != "mp3" { continue };

        use std::str::FromStr;
        let mime = mime::Mime::from_str("audio/mpeg").unwrap();

        let nugget = word.replace("*", "").replace("・", "");

        let w = manage::NewWordFromStrings {
            word,
            explanation: "".into(),
            nugget,
            narrator: narrator,
            files: vec![(temp_file_path, Some(file_name), mime)],
        };
        
        println!("{:?}", w);

        manage::create_or_update_word(&conn, w, &AUDIO_DIR).unwrap();
    }
}

fn main() {
    use clap::*;

    env_logger::init().unwrap();
    info!("Starting.");

    let matches = App::new("ganba.re word import tool")
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
    
    let path = matches.value_of("PATH").unwrap();
    let narrator = matches.value_of("NARRATOR").unwrap();

    import_batch(&path, &narrator);
}
