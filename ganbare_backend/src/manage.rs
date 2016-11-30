
use super::*;
use mime;

use std::path::PathBuf;


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

pub fn create_quiz(conn : &PgConnection, new_q: NewQuestion, mut answers: Vec<Fieldset>) -> Result<QuizQuestion> {
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
            Some(ref mut a) => { Some(audio::save(&*conn, &mut narrator, a, &mut a_bundle)?.id) },
            None => { None },
        };
        
        let mut q_bundle = None;
        for mut q_audio in &mut fieldset.q_variants {
            audio::save(&*conn, &mut narrator, &mut q_audio, &mut q_bundle)?;
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
pub struct NewWordFromStrings {
    pub word: String,
    pub explanation: String,
    pub nugget: String,
    pub narrator: String,
    pub files: Vec<(PathBuf, Option<String>, mime::Mime)>,
}

pub fn create_word(conn : &PgConnection, w: NewWordFromStrings) -> Result<Word> {
    use schema::{words};

    let nugget = skill::get_create_by_name(&*conn, &w.nugget)?;

    let mut narrator = Some(audio::get_create_narrator(&*conn, &w.narrator)?);
    let mut bundle = Some(audio::new_bundle(&*conn, &w.word)?);
    for mut file in w.files {
        audio::save(&*conn, &mut narrator, &mut file, &mut bundle)?;
    } 
    let bundle = bundle.expect("The audio bundle is initialized now.");

    let new_word = NewWord {
        word: &w.word,
        explanation: &w.explanation,
        audio_bundle: bundle.id,
        skill_nugget: nugget.id,
    };

    let word = diesel::insert(&new_word)
        .into(words::table)
        .get_result(conn)
        .chain_err(|| "Can't insert a new word!")?;

    Ok(word)
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

pub fn post_exercise(conn : &PgConnection, exercise: NewExercise, mut answers: Vec<ExerciseVariant>) -> Result<i32> {
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
