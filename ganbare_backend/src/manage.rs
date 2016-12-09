
use super::*;
use mime;

use std::path::PathBuf;
use std::path::Path;
use regex;
use reqwest;
use time;
use std;


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

pub fn create_quiz(conn : &PgConnection, new_q: NewQuestion, mut answers: Vec<Fieldset>, audio_dir: &Path) -> Result<QuizQuestion> {
    use schema::{quiz_questions, question_answers};

    info!("Creating quiz!");

    // Sanity check
    if answers.len() == 0 {
        return Err(ErrorKind::FormParseError.into());
    }
    for a in &answers {
        if a.q_variants.len() == 0 {
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

    let quiz : QuizQuestion = diesel::insert(&new_quiz)
        .into(quiz_questions::table)
        .get_result(&*conn)
        .chain_err(|| "Couldn't create a new question!")?;

    info!("{:?}", &quiz);

    let mut narrator = None;

    for fieldset in &mut answers {
        let mut a_bundle = None;
        let a_audio_id = match fieldset.answer_audio {
            Some(ref mut a) => { Some(audio::save(&*conn, &mut narrator, a, &mut a_bundle, audio_dir)?.id) },
            None => { None },
        };
        
        let mut q_bundle = None;
        for mut q_audio in &mut fieldset.q_variants {
            audio::save(&*conn, &mut narrator, &mut q_audio, &mut q_bundle, audio_dir)?;
        }
        let q_bundle = q_bundle.expect("The audio bundle is initialized now.");

        let new_answer = NewAnswer { question_id: quiz.id, answer_text: &fieldset.answer_text, a_audio_bundle: a_audio_id, q_audio_bundle: q_bundle.id };

        let answer : Answer = diesel::insert(&new_answer)
            .into(question_answers::table)
            .get_result(&*conn)
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
}

#[derive(Debug)]
pub struct NewAudio<'a> {
    pub word: String,
    pub narrator: &'a str,
    pub files: Vec<(PathBuf, Option<String>, mime::Mime)>,
}


pub fn add_audio(conn : &PgConnection, w: NewAudio, audio_dir: &Path) -> Result<AudioBundle> {

    info!("Add audio {:?}", w);

    let mut narrator = Some(audio::get_create_narrator(conn, &w.narrator)?);
    let mut bundle = Some(audio::get_create_bundle(conn, &w.word)?);

    for mut file in w.files {
        audio::save(&*conn, &mut narrator, &mut file, &mut bundle, audio_dir)?;
    } 
    let bundle = bundle.expect("The audio bundle is initialized by now.");

    Ok(bundle)
}

pub fn create_or_update_word(conn : &PgConnection, w: NewWordFromStrings, audio_dir: &Path) -> Result<Word> {
    use schema::{words};

    info!("Create word {:?}", w);

    let nugget = skill::get_create_by_name(&*conn, &w.nugget)?;

    let mut narrator = Some(audio::get_create_narrator(&*conn, &w.narrator)?);
    let mut bundle = Some(audio::get_create_bundle(&*conn, &w.word)?);

    for mut file in w.files {
        audio::save(&*conn, &mut narrator, &mut file, &mut bundle, audio_dir)?;
    } 
    let bundle = bundle.expect("The audio bundle is initialized by now.");

    let word = words::table
        .filter(words::word.eq(&w.word))
        .get_result(conn)
        .optional()?;

    if let Some(word) = word {
        return Ok(word);
    } else {
        let new_word = NewWord {
            word: &w.word,
            explanation: &w.explanation,
            audio_bundle: bundle.id,
            skill_nugget: nugget.id,
        };
    
        let word = diesel::insert(&new_word)
            .into(words::table)
            .get_result(conn)?;
        return Ok(word);
    }

}

pub fn get_question(conn : &PgConnection, id : i32) -> Result<Option<(QuizQuestion, Vec<Answer>)>> {
    if let Some((qq, aas, _)) = quiz::load_question(conn, id)? {
        Ok(Some((qq, aas)))
    } else {
        Ok(None)
    }
}

pub fn get_exercise(conn : &PgConnection, id : i32) -> Result<Option<(Exercise, Vec<ExerciseVariant>)>> {
    if let Some((qq, aas, _)) = quiz::load_exercise(conn, id)? {
        Ok(Some((qq, aas)))
    } else {
        Ok(None)
    }
}

pub fn get_word(conn : &PgConnection, id : i32) -> Result<Option<Word>> {
    Ok(schema::words::table.filter(schema::words::id.eq(id)).get_result(conn).optional()?)
}

pub fn publish_question(conn : &PgConnection, id: i32, published: bool) -> Result<()> {
    use schema::quiz_questions;
    diesel::update(quiz_questions::table
        .filter(quiz_questions::id.eq(id)))
        .set(quiz_questions::published.eq(published))
        .execute(conn)?;
    Ok(())
}

pub fn publish_exercise(conn : &PgConnection, id: i32, published: bool) -> Result<()> {
    use schema::exercises;
    diesel::update(exercises::table
        .filter(exercises::id.eq(id)))
        .set(exercises::published.eq(published))
        .execute(conn)?;
    Ok(())
}

pub fn publish_word(conn : &PgConnection, id: i32, published: bool) -> Result<()> {
    use schema::words;
    diesel::update(words::table
        .filter(words::id.eq(id)))
        .set(words::published.eq(published))
        .execute(conn)?;
    Ok(())
}

pub fn update_word(conn : &PgConnection, id: i32, item: UpdateWord) -> Result<Option<Word>> {
    use schema::words;
    let item = diesel::update(words::table
        .filter(words::id.eq(id)))
        .set(&item)
        .get_result(conn)
        .optional()?;
    Ok(item)
}

pub fn update_question(conn : &PgConnection, id: i32, item: UpdateQuestion) -> Result<Option<QuizQuestion>> {
    use schema::quiz_questions;
    let item = diesel::update(quiz_questions::table
        .filter(quiz_questions::id.eq(id)))
        .set(&item)
        .get_result(conn)
        .optional()?;
    Ok(item)
}


pub fn update_answer(conn : &PgConnection, id: i32, item: UpdateAnswer) -> Result<Option<Answer>> {
    use schema::question_answers;
    let item = diesel::update(question_answers::table
        .filter(question_answers::id.eq(id)))
        .set(&item)
        .get_result(conn)
        .optional()?;
    Ok(item)
}

pub fn post_question(conn : &PgConnection, question: NewQuizQuestion, mut answers: Vec<NewAnswer>) -> Result<i32> {
    use schema::{question_answers, quiz_questions};

    let q: QuizQuestion = diesel::insert(&question)
                .into(quiz_questions::table)
                .get_result(conn)?;

    for aa in &mut answers {
        aa.question_id = q.id;
        diesel::insert(aa)
            .into(question_answers::table)
            .execute(conn)?;
    }
    Ok(q.id)
}

pub fn post_exercise(conn: &PgConnection, exercise: NewExercise, mut answers: Vec<ExerciseVariant>) -> Result<i32> {
    use schema::{exercises, exercise_variants};

    let q: Exercise = diesel::insert(&exercise)
                .into(exercises::table)
                .get_result(conn)?;

    for aa in &mut answers {
        aa.exercise_id = q.id;
        diesel::insert(aa)
            .into(exercise_variants::table)
            .execute(conn)?;
    }
    Ok(q.id)
}

pub fn replace_audio_bundle(conn: &PgConnection, bundle_id: i32, new_bundle_id: i32) -> Result<()> {
    use schema::{words, question_answers};
    use diesel::result::TransactionError::*;

    info!("Replacing old bundle references (id {}) with new ones (id {}).", bundle_id, new_bundle_id);

    match conn.transaction(|| {

        let count = diesel::update(
                words::table.filter(words::audio_bundle.eq(bundle_id))
            ).set(words::audio_bundle.eq(new_bundle_id))
            .execute(conn)?;

        info!("{} audio bundles in words replaced with a new audio bundle.", count);
    
        let count = diesel::update(
                question_answers::table.filter(question_answers::a_audio_bundle.eq(bundle_id))
            ).set(question_answers::a_audio_bundle.eq(new_bundle_id))
            .execute(conn)?;
            
        info!("{} audio bundles in question_answers::a_audio_bundle replaced with a new audio bundle.", count);

        let count = diesel::update(
                question_answers::table.filter(question_answers::q_audio_bundle.eq(bundle_id))
            ).set(question_answers::q_audio_bundle.eq(new_bundle_id))
            .execute(conn)?;

        info!("{} audio bundles in question_answers::q_audio_bundle replaced with a new audio bundle.", count);

        Ok(())
    
    }) {
        Ok(b) => Ok(b),
        Err(e) => match e {
            CouldntCreateTransaction(e) => Err(e.into()),
            UserReturnedError(e) => Err(e),
        },
    }
}

pub fn clean_urls(conn: &PgConnection, image_dir: &Path) -> Result<Vec<String>> {
    use schema::{words, question_answers};
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
