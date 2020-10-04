
use super::*;
use mime;

use std::path::PathBuf;
use std::path::Path;


#[derive(Debug)]
pub struct Fieldset {
    pub q_variants: Vec<(PathBuf, Option<String>, mime::Mime)>,
    pub answer_audio: Option<(PathBuf, Option<String>, mime::Mime)>,
    pub answer_text: String,
}

pub struct NewQuestion {
    pub q_name: String,
    pub q_explanation: String,
    pub question_text: String,
    pub skill_nugget: String,
}

pub fn create_quiz(conn: &Connection,
                   new_q: NewQuestion,
                   mut answers: Vec<Fieldset>,
                   audio_dir: &Path)
                   -> Result<QuizQuestion> {
    use schema::{quiz_questions, question_answers};

    info!("Creating quiz!");

    // Sanity check
    if answers.is_empty() {
        warn!("Can't create a question with 0 answers!");
        return Err(ErrorKind::FormParseError.into());
    }
    for a in &answers {
        if a.q_variants.is_empty() {
            warn!("Can't create a question with 0 audio files for question!");
            return Err(ErrorKind::FormParseError.into());
        }
    }

    let nugget = skill::get_create_by_name(&*conn, &new_q.skill_nugget)?;

    let new_quiz = NewQuizQuestion {
        q_name: &new_q.q_name,
        q_explanation: &new_q.q_explanation,
        question_text: &new_q.question_text,
        skill_id: nugget.id,
        skill_level: 2, // FIXME
    };

    let quiz: QuizQuestion = diesel::insert_into(quiz_questions::table).values(&new_quiz)
        .get_result(&**conn)
        .chain_err(|| "Couldn't create a new question!")?;

    info!("{:?}", &quiz);

    let mut narrator = None;

    for fieldset in &mut answers {
        let mut a_bundle = None;
        let a_audio_id = match fieldset.answer_audio {
            Some(ref mut a) => {
                Some(audio::save(&*conn, &mut narrator, a, &mut a_bundle, audio_dir)?.id)
            }
            None => None,
        };

        let mut q_bundle = None;
        for mut q_audio in &mut fieldset.q_variants {
            audio::save(&*conn,
                        &mut narrator,
                        &mut q_audio,
                        &mut q_bundle,
                        audio_dir)?;
        }
        let q_bundle = q_bundle.expect("The audio bundle is initialized now.");

        let new_answer = NewAnswer {
            question_id: quiz.id,
            answer_text: &fieldset.answer_text,
            a_audio_bundle: a_audio_id,
            q_audio_bundle: q_bundle.id,
        };

        let answer: Answer = diesel::insert_into(question_answers::table).values(&new_answer)
            .get_result(&**conn)
            .chain_err(|| "Couldn't create a new answer!")?;

        info!("{:?}", &answer);


    }
    Ok(quiz)
}

#[derive(Debug)]
pub struct NewWordFromStrings<'a> {
    pub word: String,
    pub explanation: String,
    pub nugget: String,
    pub narrator: &'a str,
    pub files: Vec<(PathBuf, Option<String>, mime::Mime)>,
    pub skill_level: i32,
    pub priority: i32,
}

#[derive(Debug)]
pub struct NewAudio<'a> {
    pub word: String,
    pub narrator: &'a str,
    pub files: Vec<(PathBuf, Option<String>, mime::Mime)>,
}


pub fn add_audio(conn: &Connection, w: NewAudio, audio_dir: &Path) -> Result<AudioBundle> {

    info!("Add audio {:?}", w);

    let mut narrator = Some(audio::get_create_narrator(conn, w.narrator)?);
    let mut bundle = Some(audio::get_create_bundle(conn, &w.word)?);

    for mut file in w.files {
        audio::save(&*conn, &mut narrator, &mut file, &mut bundle, audio_dir)?;
    }
    let bundle = bundle.expect("The audio bundle is initialized by now.");

    Ok(bundle)
}

pub fn create_or_update_word(conn: &Connection,
                             mut w: NewWordFromStrings,
                             audio_dir: &Path)
                             -> Result<Word> {
    use schema::{words, audio_files};

    info!("Create word {:?}", w);

    let nugget = skill::get_create_by_name(&*conn, &w.nugget)?;

    let mut audio_file = None;

    match conn.transaction(|| {
        let mut narrator = Some(audio::get_create_narrator(conn, w.narrator)?);
        let mut bundle = Some(audio::get_create_bundle(conn, &w.word)?);

        for mut file in &mut w.files {
            audio_file =
                Some(audio::save(&*conn, &mut narrator, &mut file, &mut bundle, audio_dir)?);
        }
        Ok(())
    }) {
        Err(Error(ErrorKind::FileAlreadyExists(hash), ..)) => {
            audio_file = audio_files::table.filter(audio_files::file_sha2.eq(hash))
                .get_result(&**conn)
                .optional()?;
        }
        Err(e) => return Err(e),
        Ok(()) => (),
    };

    let audio_file = audio_file.expect("If we are here, everything was successful.");

    let word = words::table.filter(words::word.eq(&w.word))
        .get_result(&**conn)
        .optional()?;

    if let Some(word) = word {
        info!("The word existed already. Returning.");
        return Ok(word);
    } else {
        let new_word = NewWord {
            word: &w.word,
            explanation: &w.explanation,
            audio_bundle: audio_file.bundle_id,
            skill_nugget: nugget.id,
            skill_level: w.skill_level,
            priority: w.priority,
        };

        let word = diesel::insert_into(words::table).values(&new_word).get_result(&**conn)?;
        return Ok(word);
    }

}

pub fn get_question(conn: &Connection, id: i32) -> Result<Option<(QuizQuestion, Vec<Answer>)>> {
    if let Some((qq, aas, _)) = quiz::load_question(conn, id)? {
        Ok(Some((qq, aas)))
    } else {
        Ok(None)
    }
}

pub fn get_exercise(conn: &Connection,
                    id: i32)
                    -> Result<Option<(Exercise, Vec<ExerciseVariant>)>> {
    if let Some((qq, aas, _)) = quiz::load_exercise(conn, id)? {
        Ok(Some((qq, aas)))
    } else {
        Ok(None)
    }
}

pub fn get_word(conn: &Connection, id: i32) -> Result<Option<Word>> {
    Ok(schema::words::table.filter(schema::words::id.eq(id))
           .get_result(&**conn)
           .optional()?)
}

pub fn publish_question(conn: &Connection, id: i32, published: bool) -> Result<()> {
    use schema::quiz_questions;
    diesel::update(quiz_questions::table
        .filter(quiz_questions::id.eq(id)))
        .set(quiz_questions::published.eq(published))
        .execute(&**conn)?;
    Ok(())
}

pub fn publish_exercise(conn: &Connection, id: i32, published: bool) -> Result<()> {
    use schema::exercises;
    diesel::update(exercises::table
        .filter(exercises::id.eq(id)))
        .set(exercises::published.eq(published))
        .execute(&**conn)?;
    Ok(())
}

pub fn publish_word(conn: &Connection, id: i32, published: bool) -> Result<()> {
    use schema::words;
    diesel::update(words::table.filter(words::id.eq(id))).set(words::published.eq(published))
        .execute(&**conn)?;
    Ok(())
}

pub fn update_word(conn: &Connection,
                   id: i32,
                   mut item: UpdateWord,
                   image_dir: &Path)
                   -> Result<Option<Word>> {
    use schema::words;

    item.explanation = item.explanation.try_map(|s| sanitize_links(&s, image_dir))?;

    let item = diesel::update(words::table.filter(words::id.eq(id))).set(&item)
        .get_result(&**conn)
        .optional()?;
    Ok(item)
}

pub fn update_exercise(conn: &Connection,
                       id: i32,
                       item: UpdateExercise)
                       -> Result<Option<Exercise>> {
    use schema::exercises;
    let item = diesel::update(exercises::table.filter(exercises::id.eq(id))).set(&item)
        .get_result(&**conn)
        .optional()?;
    Ok(item)
}


pub fn update_question(conn: &Connection,
                       id: i32,
                       item: UpdateQuestion)
                       -> Result<Option<QuizQuestion>> {
    use schema::quiz_questions;
    let item = diesel::update(quiz_questions::table.filter(quiz_questions::id.eq(id))).set(&item)
        .get_result(&**conn)
        .optional()?;
    Ok(item)
}


pub fn update_answer(conn: &Connection,
                     id: i32,
                     mut item: UpdateAnswer,
                     image_dir: &Path)
                     -> Result<Option<Answer>> {
    use schema::question_answers;

    item.answer_text = item.answer_text.try_map(|s| sanitize_links(&s, image_dir))?;

    let item = diesel::update(question_answers::table.filter(question_answers::id.eq(id))).set(&item)
        .get_result(&**conn)
        .optional()?;
    Ok(item)
}

pub fn update_variant(conn: &Connection,
                      id: i32,
                      item: UpdateExerciseVariant)
                      -> Result<Option<ExerciseVariant>> {

    use schema::exercise_variants;

    let item = diesel::update(exercise_variants::table.filter(exercise_variants::id.eq(id))).set(&item)
        .get_result(&**conn)
        .optional()?;
    Ok(item)
}

pub fn remove_word(conn: &Connection, id: i32) -> Result<Option<Word>> {
    use schema::words;

    let word: Option<Word> = diesel::delete(words::table.filter(words::id.eq(id))).get_result(&**conn)
        .optional()?;

    Ok(word)
}

pub fn remove_question(conn: &Connection, id: i32) -> Result<bool> {
    use schema::{quiz_questions, question_answers};

    diesel::delete(question_answers::table.filter(question_answers::question_id.eq(id)))
        .execute(&**conn)?;

    let count =
        diesel::delete(quiz_questions::table.filter(quiz_questions::id.eq(id))).execute(&**conn)?;

    Ok(count == 1)
}

pub fn remove_exercise(conn: &Connection, id: i32) -> Result<bool> {
    use schema::{exercises, exercise_variants};

    diesel::delete(exercise_variants::table.filter(exercise_variants::exercise_id.eq(id)))
        .execute(&**conn)?;

    let count = diesel::delete(exercises::table.filter(exercises::id.eq(id))).execute(&**conn)?;

    Ok(count == 1)
}

pub fn post_question(conn: &Connection,
                     question: NewQuizQuestion,
                     mut answers: Vec<NewAnswer>)
                     -> Result<i32> {
    use schema::{question_answers, quiz_questions};

    debug!("Post question: {:?} and answers: {:?}", question, answers);

    let q: QuizQuestion =
        diesel::insert_into(quiz_questions::table).values(&question).get_result(&**conn)?;

    for aa in &mut answers {
        aa.question_id = q.id;
        diesel::insert_into(question_answers::table).values(&*aa).execute(&**conn)?;
    }
    Ok(q.id)
}

pub fn post_exercise(conn: &Connection,
                     exercise: NewExercise,
                     mut answers: Vec<ExerciseVariant>)
                     -> Result<i32> {
    use schema::{exercises, exercise_variants};

    conn.transaction(|| -> Result<i32> {

            let q: Exercise = diesel::insert_into(exercises::table).values(&exercise).get_result(&**conn)?;

            for aa in &mut answers {
                aa.exercise_id = q.id;
                diesel::insert_into(exercise_variants::table).values(&*aa).execute(&**conn)?;
            }
            Ok(q.id)

        })
        .chain_err(|| ErrorKind::from("Transaction failed"))

}

pub fn del_due_and_pending_items(conn: &Connection, user_id: i32) -> Result<()> {
    use schema::{due_items, pending_items, question_data, exercise_data, e_asked_data, q_asked_data,
                 e_answered_data, q_answered_data};
    use diesel::expression::dsl::any;

    let p = diesel::update(
            pending_items::table
                .filter(pending_items::user_id.eq(user_id).and(pending_items::pending.eq(true)))
        )
        .set(pending_items::pending.eq(false))
        .execute(&**conn)?;

    let pending: Vec<PendingItem> = pending_items::table.filter(pending_items::user_id.eq(user_id))
        .get_results(&**conn)?;

    let due_items = due_items::table.filter(due_items::user_id.eq(user_id)).select(due_items::id);

    let q = diesel::delete(question_data::table.filter(question_data::due.eq(any(due_items))))
        .execute(&**conn)?;

    let e = diesel::delete(exercise_data::table.filter(exercise_data::due.eq(any(due_items))))
        .execute(&**conn)?;

    let d =
        diesel::delete(due_items::table.filter(due_items::user_id.eq(user_id))).execute(&**conn)?;

    let mut asks = 0;
    let mut answers = 0;

    for p in &pending {

        answers += diesel::delete(e_answered_data::table.filter(e_answered_data::id.eq(p.id)))
            .execute(&**conn)?;

        answers += diesel::delete(q_answered_data::table.filter(q_answered_data::id.eq(p.id)))
            .execute(&**conn)?;

        asks +=
            diesel::delete(e_asked_data::table.filter(e_asked_data::id.eq(p.id))).execute(&**conn)?;

        asks +=
            diesel::delete(q_asked_data::table.filter(q_asked_data::id.eq(p.id))).execute(&**conn)?;

    }

    debug!("Deactivated {} pending items and deleted {} due items. ({} questions, {} exercises, \
            {} asks, {} answers)",
           p,
           d,
           q,
           e,
           asks,
           answers);

    Ok(())
}

pub fn replace_audio_bundle(conn: &Connection, bundle_id: i32, new_bundle_id: i32) -> Result<()> {
    use schema::{words, question_answers};

    info!("Replacing old bundle references (id {}) with new ones (id {}).",
          bundle_id,
          new_bundle_id);

    conn.transaction(|| {

        let count = diesel::update(
                words::table.filter(words::audio_bundle.eq(bundle_id))
            ).set(words::audio_bundle.eq(new_bundle_id))
            .execute(&**conn)?;

        info!("{} audio bundles in words replaced with a new audio bundle.",
              count);

        let count = diesel::update(
                question_answers::table.filter(question_answers::a_audio_bundle.eq(bundle_id))
            ).set(question_answers::a_audio_bundle.eq(new_bundle_id))
            .execute(&**conn)?;

        info!("{} audio bundles in question_answers::a_audio_bundle replaced with a new audio \
               bundle.",
              count);

        let count = diesel::update(
                question_answers::table.filter(question_answers::q_audio_bundle.eq(bundle_id))
            ).set(question_answers::q_audio_bundle.eq(new_bundle_id))
            .execute(&**conn)?;

        info!("{} audio bundles in question_answers::q_audio_bundle replaced with a new audio \
               bundle.",
              count);

        Ok(())

    })
}

use ureq;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {

    static ref URL_REGEX: Regex
        = Regex::new(r#"['"](https?://.*?(\.[a-zA-Z0-9]{1,4})?)['"]"#)
            .expect("<- that is a valid regex there");

    static ref EXTENSION_GUESS: Regex
        = Regex::new(r#"\.png|\.jpg|\.jpeg|\.gif"#)
            .expect("<- that is a valid regex there");

    static ref CONVERTED_LINKS: RwLock<HashMap<String, String>>
        = RwLock::new(HashMap::<String, String>::new());
}

pub fn sanitize_links(text: &str, image_dir: &Path) -> Result<String> {
    use rand::{thread_rng, Rng};
    use std::fs;
    use std::io;
    use rand::distributions::Alphanumeric;

    info!("Sanitizing text: {}", text);

    let mut result = text.to_string();
    for url_match in URL_REGEX.captures_iter(text) {

        let url =
            url_match.get(1).expect("The whole match won't match without this submatch.").as_str();

        info!("Outbound link found: {}", url);

        if CONVERTED_LINKS.read()
               .expect("If the lock is poisoned, we're screwed anyway")
               .contains_key(url) {
            let new_url =
                &CONVERTED_LINKS.read().expect("If the lock is poisoned, we're screwed anyway")
                     [url];
            result = result.replace(url, new_url);
        } else {

            info!("Downloading the link target.");
            let desanitized_url = url.replace("&amp;", "&");
            let resp = ureq::get(&desanitized_url).call();

            assert!(resp.status() < 400);

            let extension = {
                let fuzzy_guess_url: Option<&str> = EXTENSION_GUESS.captures_iter(url)
                    .next()
                    .and_then(|c| c.get(0))
                    .map(|g| g.as_str());
                let file_extension = url_match.get(2).map(|m| m.as_str());
                let content_type = resp.header("Content-Type");

                debug!("Original file extension: {:?}, Guess from URL: {:?}, Content type: {:?}",
                       file_extension,
                       fuzzy_guess_url,
                       content_type);

                match content_type {
                    Some("image/png") => ".png",
                    Some("image/jpeg") => ".jpg",
                    Some("image/gif") => ".gif",
                    Some(_) | None => {
                        file_extension.or_else(|| fuzzy_guess_url).unwrap_or(".noextension")
                    }
                }
            };

            let mut new_path = image_dir.to_owned();
            let mut filename = "%FT%H-%M-%SZ".to_string();
            filename.extend(thread_rng().sample_iter(Alphanumeric).take(10));
            filename.push_str(extension);
            filename = format!("{}", chrono::offset::Utc::now().format(&filename));
            new_path.push(&filename);

            let mut file = fs::File::create(new_path)?;
            io::copy(&mut resp.into_reader(), &mut file)?;
            info!("Saved the file to {:?}", file);
            let new_url = String::from("/api/images/") + &filename;

            result = result.replace(url, &new_url);
            CONVERTED_LINKS.write()
                .expect("If the lock is poisoned, we're screwed anyway")
                .insert(url.to_string(), new_url);
        }
        info!("Sanitized to: {}", &result);
    }
    Ok(result)
}

#[test]
fn test_sanitize_links() {
    use tempdir;
    use std::fs;

    let tempdir = tempdir::TempDir::new("").unwrap();
    assert_eq!(fs::read_dir(tempdir.path()).unwrap().count(), 0);
    let result = sanitize_links("Testing \"http://static4.depositphotos.\
                        com/1016045/326/i/950/depositphotos_3267906-stock-photo-cool-emoticon.\
                        jpg\" testing",
                                tempdir.path())
            .unwrap();
    assert_eq!(fs::read_dir(tempdir.path()).unwrap().count(), 1);
    let result2 = sanitize_links("Testing \"http://static4.depositphotos.\
                        com/1016045/326/i/950/depositphotos_3267906-stock-photo-cool-emoticon.\
                        jpg\" testing",
                                 tempdir.path())
            .unwrap();
    assert_eq!(fs::read_dir(tempdir.path()).unwrap().count(), 1);
    assert_eq!(result.len(), 64);
    assert_eq!(result, result2);
    let result3 = sanitize_links("Testing \"https://c2.staticflickr.\
                                  com/2/1216/1408154388_b34a66bdcf.jpg\" testing",
                                 tempdir.path())
            .unwrap();
    assert_eq!(fs::read_dir(tempdir.path()).unwrap().count(), 2);
    assert_eq!(result3.len(), 64);
    assert_ne!(result, result3);
    tempdir.close().unwrap();
}
