
    use super::*;
    use std::path::PathBuf;
    use std::path::Path;
    use rand::thread_rng;
    use time;
    use std::fs;
    use std::mem;
    use mime;

fn save_file(path: &mut PathBuf, orig_filename: &str, audio_dir: &Path) -> Result<()> {
    info!("Saving file {:?}", &orig_filename);
    use rand::Rng;
    let mut new_path = audio_dir.to_owned();
    let mut filename = "%FT%H-%M-%SZ".to_string();
    filename.extend(thread_rng().gen_ascii_chars().take(10));
    filename.push_str(".");
    filename.push_str(Path::new(orig_filename).extension().and_then(|s| s.to_str()).unwrap_or("noextension"));
    new_path.push(time::strftime(&filename, &time::now()).unwrap());
    info!("Renaming {:?} to {:?}", &*path, &new_path);
    fs::rename(&*path, &new_path).chain_err(|| "Can't rename the audio file.")?;
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

pub fn del_narrator(conn : &PgConnection, id: i32) -> Result<bool> {
    use schema::{narrators, audio_files};
    use diesel::result::TransactionError::*;

    match conn.transaction(|| {

        info!("Deleting audio_files with narrators_id {:?}", id);

        let audio_files_count = diesel::delete(audio_files::table
            .filter(audio_files::narrators_id.eq(id)))
            .execute(conn)?;

        info!("Rows deleted {:?}", audio_files_count);
    
        info!("Deleting narrator with id {:?}", id);
        
        let narrators_count = diesel::delete(narrators::table
            .filter(narrators::id.eq(id)))
            .execute(conn)?;
    
        info!("Rows deleted {:?}", narrators_count);

        Ok(narrators_count == 1)
    
    }) {
        Ok(b) => Ok(b),
        Err(e) => match e {
            CouldntCreateTransaction(e) => Err(e.into()),
            UserReturnedError(e) => Err(e),
        },
    }
}

pub fn merge_narrator(conn: &PgConnection, narrator_id: i32, new_narrator_id: i32) -> Result<()> {
    use schema::{audio_files, narrators};
    use diesel::result::TransactionError::*;

    info!("Replacing old narrator references (id {}) with new ones (id {}).", narrator_id, new_narrator_id);

    match conn.transaction(|| {

        let count = diesel::update(
                audio_files::table.filter(audio_files::narrators_id.eq(narrator_id))
            ).set(audio_files::narrators_id.eq(new_narrator_id))
            .execute(conn)?;

        info!("{} narrators in audio files replaced with a new audio bundle.", count);

        diesel::delete(narrators::table.filter(narrators::id.eq(narrator_id))).execute(conn)?;

        Ok(())
    
    }) {
        Ok(b) => Ok(b),
        Err(e) => match e {
            CouldntCreateTransaction(e) => Err(e.into()),
            UserReturnedError(e) => Err(e),
        },
    }
}

pub fn merge_audio_bundle(conn: &PgConnection, bundle_id: i32, new_bundle_id: i32) -> Result<()> {
    use schema::{audio_files, audio_bundles};
    use diesel::result::TransactionError::*;

    info!("Replacing old bundle references (id {}) with new ones (id {}).", bundle_id, new_bundle_id);

    match conn.transaction(|| {

        // updating audio_files
        let count = diesel::update(
                audio_files::table.filter(audio_files::bundle_id.eq(bundle_id))
            ).set(audio_files::bundle_id.eq(new_bundle_id))
            .execute(conn)?;

        info!("{} bundles in audio files replaced with a new audio bundle.", count);

        // updating words & questions
        manage::replace_audio_bundle(conn, bundle_id, new_bundle_id)?;

        diesel::delete(audio_bundles::table.filter(audio_bundles::id.eq(bundle_id))).execute(conn)?;
        Ok(())
    
    }) {
        Ok(b) => Ok(b),
        Err(e) => match e {
            CouldntCreateTransaction(e) => Err(e.into()),
            UserReturnedError(e) => Err(e),
        },
    }
}

pub fn del_bundle(conn : &PgConnection, id: i32) -> Result<bool> {
    use schema::{audio_bundles, audio_files, question_answers, words};
    use diesel::result::TransactionError::*;

    match conn.transaction(|| {

        // To avoid deleting words and questions, let's find a replacement bundle for all the things that depend on this!

        let bundle: AudioBundle = audio_bundles::table
            .filter(audio_bundles::id.eq(id))
            .get_result(conn)?;

        let replacement_bundles = get_bundles_by_name(conn, &bundle.listname)?;
        for bundle in replacement_bundles {
            if bundle.id != id { // A proper replacement found!
                manage::replace_audio_bundle(conn, id, bundle.id)?;
            }
        }

        info!("Deleting audio_files with bundle_id {:?}", id);

        let count = diesel::delete(audio_files::table
            .filter(audio_files::bundle_id.eq(id)))
            .execute(conn)?;

        info!("Rows deleted {:?}", count);
    
        info!("Deleting words with bundle_id {:?}", id);

        let count = diesel::delete(words::table
            .filter(words::audio_bundle.eq(id)))
            .execute(conn)?;

        info!("Rows deleted {:?}", count);
    
        info!("Deleting q_answers with bundle_id {:?}", id);

        let count = diesel::delete(question_answers::table
            .filter(question_answers::a_audio_bundle.eq(id)))
            .execute(conn)?;

        info!("Rows deleted {:?}", count);
    
        info!("Deleting q_answers with bundle_id {:?}", id);

        let count = diesel::delete(question_answers::table
            .filter(question_answers::q_audio_bundle.eq(id)))
            .execute(conn)?;

        info!("Rows deleted {:?}", count);
    
        info!("Deleting bundle with id {:?}", id);
        
        let count = diesel::delete(audio_bundles::table
            .filter(audio_bundles::id.eq(id)))
            .execute(conn)?;
    
        info!("Rows deleted {:?}", count);

        Ok(count == 1)
    
    }) {
        Ok(b) => Ok(b),
        Err(e) => match e {
            CouldntCreateTransaction(e) => Err(e.into()),
            UserReturnedError(e) => Err(e),
        },
    }
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

pub fn change_bundle_name(conn: &PgConnection, id: i32, new_name: &str) -> Result<Option<AudioBundle>> {
    use schema::audio_bundles;

    let bundle: Option<AudioBundle> = diesel::update(audio_bundles::table
        .filter(audio_bundles::id.eq(id)))
        .set(audio_bundles::listname.eq(new_name))
        .get_result(conn)
        .optional()?;

    Ok(bundle)
}

pub fn change_narrator_name(conn: &PgConnection, id: i32, new_name: &str) -> Result<Option<Narrator>> {
    use schema::narrators;

    let narrator: Option<Narrator> = diesel::update(narrators::table
        .filter(narrators::id.eq(id)))
        .set(narrators::name.eq(new_name))
        .get_result(conn)
        .optional()?;

    Ok(narrator)
}

pub fn get_create_bundle(conn : &PgConnection, listname: &str) -> Result<AudioBundle> {
    use schema::audio_bundles;

    let bundle : Option<AudioBundle> = {
         audio_bundles::table
            .filter(audio_bundles::listname.eq(listname))
            .get_result(conn)
            .optional()?
    };

    Ok(match bundle {
        Some(bundle) => bundle,
        None => {
            diesel::insert(&NewAudioBundle{ listname })
                .into(audio_bundles::table)
                .get_result(conn)?
        }
    })
}

pub fn get_bundles_by_name(conn : &PgConnection, listname: &str) -> Result<Vec<AudioBundle>> {
    use schema::audio_bundles;

    let bundle : Vec<AudioBundle> = {
         audio_bundles::table
            .filter(audio_bundles::listname.eq(listname))
            .get_results(conn)?
    };

    Ok(bundle)
}

pub fn get_narrators_by_name(conn : &PgConnection, name: &str) -> Result<Vec<Narrator>> {
    use schema::narrators;

    let narr : Vec<Narrator> = {
         narrators::table
            .filter(narrators::name.eq(name))
            .get_results(conn)?
    };

    Ok(narr)
}


pub fn save(conn : &PgConnection, mut narrator: &mut Option<Narrator>, file: &mut (PathBuf, Option<String>, mime::Mime), bundle: &mut Option<AudioBundle>, audio_dir: &Path)
    -> Result<AudioFile> {
    use schema::{audio_files};

    save_file(&mut file.0, file.1.as_ref().map(|s| s.as_str()).unwrap_or(""), audio_dir)?;

    let bundle_id = if let &mut Some(ref bundle) = bundle {
            bundle.id
        } else {
            let new_bundle = new_bundle(&*conn, "")?;
            let bundle_id = new_bundle.id;
            *bundle = Some(new_bundle);
            bundle_id
        };

    let file_path = file.0.file_name().expect("We just set the file name").to_str().expect("this is an ascii path");
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

pub fn load_all_from_bundles(conn : &PgConnection, bundles: &[AudioBundle]) -> Result<Vec<Vec<AudioFile>>> {

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

pub fn load_all_from_bundle(conn : &PgConnection, bundle_id: i32) -> Result<Vec<AudioFile>> {
    use schema::audio_files;

    let q_audio_files : Vec<AudioFile> = audio_files::table
        .filter(audio_files::bundle_id.eq(bundle_id))
        .get_results(&*conn)
        .chain_err(|| "Can't load quiz!")?;
    Ok(q_audio_files)
}

pub fn load_random_from_bundle(conn : &PgConnection, bundle_id: i32) -> Result<AudioFile> {
    use schema::audio_files;
    use rand::{Rng, thread_rng};

    let mut q_audio_files : Vec<AudioFile> = audio_files::table
        .filter(audio_files::bundle_id.eq(bundle_id))
        .get_results(&*conn)
        .chain_err(|| "Can't load quiz!")?;
    
    let random_index = thread_rng().gen_range(0, q_audio_files.len()); // Panics if q_audio_files.len() == 0
    let audio_file = q_audio_files.swap_remove(random_index);
    Ok(audio_file)
}

pub fn get_all_bundles(conn : &PgConnection) -> Result<Vec<(AudioBundle, Vec<AudioFile>)>> {
    use schema::{audio_bundles};
    let bundles: Vec<AudioBundle> = audio_bundles::table.get_results(conn)?;

    // FIXME checking this special case until the panicking bug in Diesel is fixed
    let audio_files = if bundles.len() > 0 {
        AudioFile::belonging_to(&bundles).load::<AudioFile>(conn)?.grouped_by(&bundles)
    } else { vec![] };
    let all = bundles.into_iter().zip(audio_files).collect();
    Ok(all)
}

pub fn get_narrators(conn : &PgConnection) -> Result<Vec<Narrator>> {
    use schema::{narrators};
    let narrators: Vec<Narrator> = narrators::table.get_results(conn)?;
    Ok(narrators)
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

pub fn get_all_files(conn : &PgConnection) -> Result<Vec<(String, mime::Mime)>> {
    use schema::audio_files::dsl::*;

    let files : Vec<AudioFile> = audio_files
        .get_results(conn)?;

    let files = files
        .into_iter()
        .map(|f| (f.file_path, f.mime.parse().expect("The mimetype from the database should be always valid.")))
        .collect();

    Ok(files)
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
