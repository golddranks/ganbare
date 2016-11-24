#![feature(field_init_shorthand)]

extern crate ganbare_backend;

#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate mime;
extern crate unicode_normalization;
extern crate tempdir;
#[macro_use]  extern crate lazy_static;

use unicode_normalization::UnicodeNormalization;
use ganbare_backend::*;

lazy_static! {

    static ref DATABASE_URL : String = { dotenv::dotenv().ok(); std::env::var("GANBARE_DATABASE_URL")
        .expect("GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)")};

}

fn import_batch(path: &str) {
    let conn = db_connect(&*DATABASE_URL).unwrap();

    let files = std::fs::read_dir(path).unwrap();

    let tmp_dir = tempdir::TempDir::new("ganbare_backend_import_tool").expect("create temp dir");
    for f in files {
        let f = f.unwrap();
        let path = f.path();
        let extension = if let Some(Some(e)) = path.extension().map(|s|s.to_str()) { e } else { continue }.to_string();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let word = path.file_stem().unwrap().to_str().unwrap().nfc().collect::<String>();
        let containing_dir = path.parent().unwrap().strip_prefix(
                                                                    path.parent().unwrap().parent().unwrap()
                                                                ).unwrap().to_str().unwrap().to_string();

        let temp_file_path = tmp_dir.path().join(f.file_name());

        std::fs::copy(path, &temp_file_path).expect("copying files");

        if extension != "mp3" { continue };

        use std::str::FromStr;
        let mime = mime::Mime::from_str("audio/mpeg").unwrap();

        let nugget = word.replace("*", "").replace("・", "");

        let w = NewWordFromStrings {
            word,
            explanation: "".into(),
            nugget,
            narrator: containing_dir,
            files: vec![(temp_file_path, Some(file_name), mime)],
        };
        
        println!("{:?}", w);

        create_word(&conn, w).unwrap();
    }

}


fn main() {
    use clap::*;

    let matches = App::new("ganba.re word import tool")
        .arg(Arg::with_name("PATH")
                                .index(1)
                                .required(true)
                                .value_name("PATH")
                                .help("The path to the input files")
                                .takes_value(true))
        .version(crate_version!())
        .get_matches();
    
    let path = matches.value_of("PATH").unwrap();

    import_batch(&path);
}
