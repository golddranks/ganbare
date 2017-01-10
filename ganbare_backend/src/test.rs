use super::*;
use quiz::{Answered, Quiz, QuizSerialized};
use rustc_serialize::json;


fn unpend_pending_test_item(conn: &PgConnection, answer_enum: &Answered) -> Result<()> {
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

    debug!("Pending test item unpended.");

    Ok(())
}

pub fn get_new_quiz_pretest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);


    let quizes = vec![
        QuizSerialized::Word("あ・か", 3608),
        QuizSerialized::Word("あか・", 3598),
        QuizSerialized::Question("あか", 3598, 1),
        QuizSerialized::Exercise("あ・か", 3608),
        QuizSerialized::Word("あ・き", 1),
        QuizSerialized::Word("あき", 1),
        QuizSerialized::Question("あき", 1, 1),
        QuizSerialized::Exercise("あ・き", 1),
        QuizSerialized::Word("あ・く", 1),
        QuizSerialized::Word("あく", 1),
        QuizSerialized::Question("あく", 1, 1),
        QuizSerialized::Exercise("あ・く", 1),
        QuizSerialized::Word("あ・か", 1),
        QuizSerialized::Word("あか・", 1),
        QuizSerialized::Question("あか", 1, 1),
        QuizSerialized::Exercise("あ・か", 1),
        QuizSerialized::Word("あ・き", 1),
        QuizSerialized::Word("あき", 1),
        QuizSerialized::Question("あき", 1, 1),
        QuizSerialized::Exercise("あ・き", 1),
        QuizSerialized::Word("あ・く", 1),
        QuizSerialized::Word("あく", 1),
        QuizSerialized::Question("あく", 1, 1),
        QuizSerialized::Exercise("あ・く", 1),
    ];


    if number == quizes.len() {
        event::set_done(&conn, &event.name, &user)?;
        return Ok(None)
    }

    let mut quiz = quiz::test_item(conn, user, &quizes[number])?;

    if let Quiz::E(ref mut e) = quiz {
        e.must_record = true;
        e.event_name = Some("pretest");
    }

    Ok(Some(quiz))
}


pub fn get_next_quiz_pretest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{
    unpend_pending_test_item(conn, &answer_enum)?;

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
        QuizSerialized::Word("あか・", 1),
        QuizSerialized::Word("あ・か", 1),
        QuizSerialized::Question("あか", 1, 1),
        QuizSerialized::Exercise("あか", 1),
    ];

    let mut quiz = quiz::test_item(conn, user, &quizes[number])?;

    if let Quiz::E(ref mut e) = quiz {
        e.must_record = true;
        e.event_name = Some("posttest");
    }

    Ok(Some(quiz))
}


pub fn get_next_quiz_posttest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{

    get_new_quiz_pretest(conn, user, event)
}

