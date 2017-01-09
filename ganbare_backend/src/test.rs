use super::*;
use quiz::{Answered, Quiz, QuizType};
use rustc_serialize::json;

/* PUBLIC APIS */

fn unpend_pending_item(conn: &PgConnection, answer_enum: &Answered) -> Result<()> {
    use schema::pending_items;

    let answered_id = match answer_enum {
        &Answered::E(ref a) => a.id,
        &Answered::Q(ref a) => a.id,
        &Answered::W(ref a) => a.id,
    };

    let mut pending_item: PendingItem = pending_items::table
        .filter(pending_items::id.eq(answered_id))
        .get_result(conn)?;
    pending_item.pending = false;
    let _ : PendingItem = pending_item.save_changes(conn)?;

    debug!("Pending item unpended.");

    Ok(())
}

enum QuizStr {
    Word(&'static str),
    Question(&'static str),
    Exercise(&'static str),
}

fn get_quiz_type(conn: &PgConnection, quiz_str: &QuizStr) -> Result<QuizType> {
    Ok(match quiz_str {
        &QuizStr::Word(ref s) => QuizType::Word(quiz::get_word_id(conn, s).chain_err(|| format!("Word {} not found", s))?),
        &QuizStr::Question(ref s) => QuizType::Question(quiz::get_question_id(conn, s).chain_err(|| format!("Question {} not found", s))?),
        &QuizStr::Exercise(ref s) => QuizType::Exercise(quiz::get_exercise_id(conn, s).chain_err(|| format!("Exercise {} not found", s))?),
    })
}

pub fn get_new_quiz_pretest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);


    let quizes = vec![
        QuizStr::Word("あ・か"),
        QuizStr::Word("あか・"),
        QuizStr::Question("あか"),
        QuizStr::Exercise("あか"),
        QuizStr::Word("あ・き"),
        QuizStr::Word("あき"),
        QuizStr::Question("あき"),
        QuizStr::Exercise("あき"),
        QuizStr::Word("あ・く"),
        QuizStr::Word("あく"),
        QuizStr::Question("あく"),
        QuizStr::Exercise("あく"),
        QuizStr::Word("あ・か"),
        QuizStr::Word("あか・"),
        QuizStr::Question("あか"),
        QuizStr::Exercise("あか"),
        QuizStr::Word("あ・き"),
        QuizStr::Word("あき"),
        QuizStr::Question("あき"),
        QuizStr::Exercise("あき"),
        QuizStr::Word("あ・く"),
        QuizStr::Word("あく"),
        QuizStr::Question("あく"),
        QuizStr::Exercise("あく"),
    ];


    if number == quizes.len() {
        event::set_done(&conn, &event.name, &user)?;
        return Ok(None)
    }

    let quiz_type_id = get_quiz_type(conn, &quizes[number])?;

    let mut quiz = quiz::return_some_quiz(conn, user, quiz_type_id);

    if let Ok(Some(Quiz::E(ref mut e))) = quiz {
        e.must_record = true;
        e.event_name = Some("pretest");
    }

    quiz
}


pub fn get_next_quiz_pretest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{
    unpend_pending_item(conn, &answer_enum)?;

    let mut number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);
    let answer_key = "answer_".to_string() + &number.to_string();
    let answer_json = json::encode(&answer_enum).unwrap();
    event::save_userdata(conn, event, user, Some(answer_key.as_str()), answer_json.as_str())?;
    number += 1;
    event::save_userdata(conn, event, user, Some("quiz_number"), &number.to_string())?;
    get_new_quiz_pretest(conn, user, event)
}


pub fn get_new_quiz_posttest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);

    let quizes = vec![
        QuizStr::Word("あか・"),
        QuizStr::Word("あ・か"),
        QuizStr::Question("あか"),
        QuizStr::Exercise("あか"),
    ];

    let quiz_type_id = get_quiz_type(conn, &quizes[number])?;

    let mut quiz = quiz::return_some_quiz(conn, user, quiz_type_id);

    if let Ok(Some(Quiz::E(ref mut e))) = quiz {
        e.must_record = true;
        e.event_name = Some("posttest");
    }

    quiz
}


pub fn get_next_quiz_posttest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{

    get_new_quiz_pretest(conn, user, event)
}

