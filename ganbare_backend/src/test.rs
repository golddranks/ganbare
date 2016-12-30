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


pub fn get_new_quiz_pretest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);

    let quizes = vec![
        QuizType::Word(1),
        QuizType::Word(5),
        QuizType::Question(10),
        QuizType::Exercise(34),
        QuizType::Word(3),
        QuizType::Word(7),
        QuizType::Question(1),
        QuizType::Exercise(1),
        QuizType::Word(1),
        QuizType::Word(5),
        QuizType::Question(10),
        QuizType::Exercise(34),
        QuizType::Word(3),
        QuizType::Word(7),
        QuizType::Question(1),
        QuizType::Exercise(1),
        QuizType::Word(1),
        QuizType::Word(5),
        QuizType::Question(10),
        QuizType::Exercise(34),
        QuizType::Word(3),
        QuizType::Word(7),
        QuizType::Question(1),
        QuizType::Exercise(1),
    ];

    if number == quizes.len() {
        event::set_done(&conn, &event.name, &user)?;
        return Ok(None)
    }

    quiz::return_some_quiz(conn, user, quizes[number])
}


pub fn get_next_quiz_pretest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{
    unpend_pending_item(conn, &answer_enum)?;

    let mut number = event::get_userdata(conn, event, user, "number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);
    let answer_key = "answer_".to_string() + &number.to_string();
    let answer_json = json::encode(&answer_enum).unwrap();
    event::save_userdata(conn, event, user, Some(answer_key.as_str()), answer_json.as_str())?;
    number += 1;
    event::save_userdata(conn, event, user, Some("number"), &number.to_string())?;
    get_new_quiz_pretest(conn, user, event)
}


pub fn get_new_quiz_posttest(conn : &PgConnection, user : &User, event: &Event) -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "number")?.and_then(|d| d.data.parse::<usize>().ok()).unwrap_or(0);

    let quizes = vec![
        QuizType::Word(1),
        QuizType::Word(4),
        QuizType::Question(1),
        QuizType::Exercise(4),
    ];

    quiz::return_some_quiz(conn, user, quizes[number])
}


pub fn get_next_quiz_posttest(conn : &PgConnection, user : &User, answer_enum: Answered, event: &Event)
    -> Result<Option<Quiz>>
{

    get_new_quiz_pretest(conn, user, event)
}

