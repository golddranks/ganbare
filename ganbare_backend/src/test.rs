use super::*;
use quiz::{Answered, Quiz, QuizSerialized};
use rustc_serialize::json;


fn save_answer_test_item(conn: &PgConnection, user: &User, event: &Event, answer_enum: &Answered) -> Result<()> {
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

    event::save_userdata(conn, event, user, Some("pending_test_item"), "0")?;

    let mut number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);
    let answer_key = "answer_".to_string() + &number.to_string();
    let answer_json = json::encode(&answer_enum).unwrap();
    event::save_userdata(conn, event, user, Some(answer_key.as_str()), answer_json.as_str())?;
    number += 1;
    event::save_userdata(conn, event, user, Some("quiz_number"), &number.to_string())?;

    debug!("Pending test item unpended & answer saved.");

    Ok(())
}

fn get_new_quiz_test(conn : &PgConnection, user : &User, event: &Event, quizes: &Vec<QuizSerialized>) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "quiz_number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);
    let pending_test_item = event::get_userdata(conn, event, user, "pending_test_item")?.and_then(|d| d.data.parse::<i32>().ok()).unwrap_or(0);

    let quiz;

    if pending_test_item == 0 { // We can show a new item, since to old one was answered to
    
        if number == quizes.len() {
            event::set_done(&conn, &event.name, &user)?;
            return Ok(None)
        }
    
        quiz = quiz::test_item(conn, user, &quizes[number])?;
    
    } else {
        use schema::pending_items;
        let pending_item: PendingItem = pending_items::table
            .filter(pending_items::id.eq(pending_test_item))
            .get_result(conn)?;

        quiz = quiz::pi_to_quiz(conn, &pending_item)?;
    }

    Ok(Some(quiz))
}

pub fn get_new_quiz_pretest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

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

    let mut quiz = get_new_quiz_test(conn, user, event, &quizes)?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = Some("pretest");
    }

    Ok(quiz)
}

pub fn get_next_quiz_pretest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{
    save_answer_test_item(conn, user, event, &answer_enum)?;
    get_new_quiz_pretest(conn, user, event)
}


pub fn get_new_quiz_posttest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let quizes = vec![
        QuizSerialized::Word("あか・", 1),
        QuizSerialized::Word("あ・か", 1),
        QuizSerialized::Question("あか", 1, 1),
        QuizSerialized::Exercise("あか", 1),
    ];

    let mut quiz = get_new_quiz_test(conn, user, event, &quizes)?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = Some("posttest");
    }

    Ok(quiz)
}


pub fn get_next_quiz_posttest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{
    save_answer_test_item(conn, user, event, &answer_enum)?;
    get_new_quiz_pretest(conn, user, event)
}

