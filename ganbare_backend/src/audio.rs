
    use super::*;
    use std::path::PathBuf;
    use std::path::Path;
    use rand::thread_rng;
    use time;
    use std::fs;
    use std::mem;
    use mime;

fn save_file(path: &mut PathBuf, orig_filename: &str) -> Result<()> {
    use rand::Rng;
    let mut new_path = PathBuf::from("audio/");
    let mut filename = "%FT%H-%M-%SZ".to_string();
    filename.extend(thread_rng().gen_ascii_chars().take(10));
    filename.push_str(".");
    filename.push_str(Path::new(orig_filename).extension().and_then(|s| s.to_str()).unwrap_or("noextension"));
    new_path.push(time::strftime(&filename, &time::now()).unwrap());
    fs::rename(&*path, &new_path)?;
    mem::swap(path, &mut new_path);
    Ok(())
}

pub fn get_create_narrator(conn : &PgConnection, mut name: &str) -> Result<Narrator> {
    use schema::narrators;

    let narrator : Option<Narrator> = if name == "" {
        name = "anonymous";
        None
    } else {
         narrators::table
            .filter(narrators::name.eq(name))
            .get_result(&*conn)
            .optional()
            .chain_err(|| "Database error with narrators!")?
    };


    Ok(match narrator {
        Some(narrator) => narrator,
        None => {
            diesel::insert(&NewNarrator{ name })
                .into(narrators::table)
                .get_result(&*conn)
                .chain_err(|| "Database error!")?
        }
    })
}

fn default_narrator_id(conn: &PgConnection, opt_narrator: &mut Option<Narrator>) -> Result<i32> {
    use schema::narrators;

    if let Some(ref narrator) = *opt_narrator {
        Ok(narrator.id)
    } else {

        let new_narrator : Narrator = diesel::insert(&NewNarrator { name: "anonymous" })
            .into(narrators::table)
            .get_result(conn)
            .chain_err(|| "Couldn't create a new narrator!")?;

        info!("{:?}", &new_narrator);
        let narr_id = new_narrator.id;
        *opt_narrator = Some(new_narrator);
        Ok(narr_id)
    }
}

pub fn new_bundle(conn : &PgConnection, name: &str) -> Result<AudioBundle> {
    use schema::{audio_bundles};
        let bundle: AudioBundle = diesel::insert(&NewAudioBundle { listname: name })
            .into(audio_bundles::table)
            .get_result(&*conn)
            .chain_err(|| "Can't insert a new audio bundle!")?;
        
        info!("{:?}", bundle);

        Ok(bundle)
}


pub fn save(conn : &PgConnection, mut narrator: &mut Option<Narrator>, file: &mut (PathBuf, Option<String>, mime::Mime), bundle: &mut Option<AudioBundle>) -> Result<AudioFile> {
    use schema::{audio_files};

    save_file(&mut file.0, file.1.as_ref().map(|s| s.as_str()).unwrap_or(""))?;

    let bundle_id = if let &mut Some(ref bundle) = bundle {
            bundle.id
        } else {
            let new_bundle = new_bundle(&*conn, "")?;
            let bundle_id = new_bundle.id;
            *bundle = Some(new_bundle);
            bundle_id
        };

    let file_path = file.0.to_str().expect("this is an ascii path");
    let mime = &format!("{}", file.2);
    let narrators_id = default_narrator_id(&*conn, &mut narrator)?;
    let new_q_audio = NewAudioFile {narrators_id, bundle_id, file_path, mime};

    let audio_file : AudioFile = diesel::insert(&new_q_audio)
        .into(audio_files::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new audio file!")?;

    info!("{:?}", &audio_file);

    

    Ok(audio_file)
}

pub fn load_from_bundles(conn : &PgConnection, bundles: &[AudioBundle]) -> Result<Vec<Vec<AudioFile>>> {

    let q_audio_files : Vec<Vec<AudioFile>> = AudioFile::belonging_to(&*bundles)
        .load(&*conn)
        .chain_err(|| "Can't load quiz!")?
        .grouped_by(&*bundles);

    for q in &q_audio_files { // Sanity check
        if q.len() == 0 {
            return Err(ErrorKind::DatabaseOdd("Bug: Audio bundles should always have more than zero members when created.").into());
        }
    };
    Ok(q_audio_files)
}

pub fn load_from_bundle(conn : &PgConnection, bundle_id: i32) -> Result<Vec<AudioFile>> {
    use schema::audio_files;

    let q_audio_files : Vec<AudioFile> = audio_files::table
        .filter(audio_files::bundle_id.eq(bundle_id))
        .get_results(&*conn)
        .chain_err(|| "Can't load quiz!")?;
    Ok(q_audio_files)
}

pub fn get_bundles(conn : &PgConnection) -> Result<Vec<(AudioBundle, Vec<AudioFile>)>> {
    use schema::{audio_bundles};
    let bundles: Vec<AudioBundle> = audio_bundles::table.get_results(conn)?;

    // FIXME checking this special case until the panicking bug in Diesel is fixed
    let audio_files = if bundles.len() > 0 {
        AudioFile::belonging_to(&bundles).load::<AudioFile>(conn)?.grouped_by(&bundles)
    } else { vec![] };
    let all = bundles.into_iter().zip(audio_files).collect();
    Ok(all)
}

pub fn get_file(conn : &PgConnection, line_id : i32) -> Result<(String, mime::Mime)> {
    use schema::audio_files::dsl::*;
    use diesel::result::Error::NotFound;

    let file : AudioFile = audio_files
        .filter(id.eq(line_id))
        .get_result(&*conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::FileNotFound),
                e => e.caused_err(|| "Couldn't get the file!"),
        })?;

    Ok((file.file_path, file.mime.parse().expect("The mimetype from the database should be always valid.")))
}

pub fn for_quiz(conn : &PgConnection, user: &User, pending_id: i32) -> Result<(String, mime::Mime)> {
    use schema::audio_files;
    use schema::pending_items;
    use diesel::result::Error::NotFound;

    let (file, _) : (AudioFile, PendingItem) = audio_files::table
        .inner_join(pending_items::table)
        .filter(pending_items::id.eq(pending_id))
        .filter(pending_items::user_id.eq(user.id))
        .filter(pending_items::pending.eq(true))
        .get_result(&*conn)
        .map_err(|e| match e {
                e @ NotFound => e.caused_err(|| ErrorKind::FileNotFound),
                e => e.caused_err(|| "Couldn't get the file!"),
        })?;
    Ok((file.file_path, file.mime.parse().expect("The mimetype from the database should be always valid.")))
       
}
