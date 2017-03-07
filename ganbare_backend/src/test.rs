use super::*;
use quiz::{Answered, Quiz, QuizSerialized};

pub fn save_answer_test_item(conn: &Connection,
                         user_id: i32,
                         event: &Event,
                         answer_enum: &Answered)
                         -> Result<()> {
    use schema::pending_items;

    let answered_id = match *answer_enum {
        Answered::E(ref a) => a.id,
        Answered::Q(ref a) => a.id,
        Answered::W(ref a) => a.id,
    };

    let mut pending_item: PendingItem =
        pending_items::table.filter(pending_items::id.eq(answered_id))
            .get_result(&**conn)?;

    assert!(pending_item.test_item);

    if !pending_item.pending {
        info!("The user tried to answer to the same question twice! Ignoring the later answer.");
        return Ok(());
    }
    pending_item.pending = false;
    let _: PendingItem = pending_item.save_changes(&**conn)?;

    event::save_userdata(conn, event, user_id, Some("pending_test_item"), "0")?;

    let mut number = event::get_userdata(conn, event, user_id, "quiz_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);
    let answer_key = "answer_".to_string() + &number.to_string();
    let answer_json = serde_json::to_string(&answer_enum).unwrap();
    event::save_userdata(conn,
                         event,
                         user_id,
                         Some(answer_key.as_str()),
                         answer_json.as_str())?;
    number += 1;
    event::save_userdata(conn,
                         event,
                         user_id,
                         Some("quiz_number"),
                         &number.to_string())?;

    debug!("Pending test item unpended & answer saved.");

    Ok(())
}

pub fn get_new_quiz_test(conn: &Connection,
                     user_id: i32,
                     event: &Event,
                     quizes: &[QuizSerialized])
                     -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user_id, "quiz_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);
    let pending_test_item = event::get_userdata(conn, event, user_id, "pending_test_item")
        ?
        .and_then(|d| d.data.parse::<i32>().ok())
        .unwrap_or(0);

    let quiz = if pending_test_item == 0 {
        // We can show a new item, since to old one was answered to

        if number == quizes.len() {
            event::set_done(conn, &event.name, user_id)?;
            return Ok(None);
        }

        let (quiz, pi_id) = quiz::test_item(conn, user_id, &quizes[number])?;
        event::save_userdata(conn,
                             event,
                             user_id,
                             Some("pending_test_item"),
                             &format!("{}", pi_id))?;
        println!("New question number {}", number);
        quiz

    } else {
        use schema::pending_items;
        let pending_item: PendingItem =
            pending_items::table.filter(pending_items::id.eq(pending_test_item))
                .get_result(&**conn)?;

        assert!(pending_item.test_item);

        println!("Returning a pending item ID {}.", pending_item.id);

        quiz::penditem_to_quiz(conn, &pending_item)?
    };

    Ok(Some(quiz))
}

#[derive(Serialize)]
pub struct RetellingJson {
    img_src: String,
    audio_src: String,
}

pub fn save_retelling(conn: &Connection,
                                   user_id: i32,
                                   event: &Event)
                                   -> Result<usize> {
    let number = event::get_userdata(conn, event, user_id, "retelling_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0) + 1;
    event::save_userdata(conn,
                         event,
                         user_id,
                         Some("retelling_number"),
                         &number.to_string())?;
    Ok(number)
}

pub fn get_new_retelling(conn: &Connection,
                     user_id: i32,
                     event: &Event,
                     retellings: &[(&'static str, &'static str)])
                     -> Result<Option<RetellingJson>> {

    let number = event::get_userdata(conn, event, user_id, "retelling_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);

    if number == retellings.len() {
        event::set_done(conn, &event.name, user_id)?;
        return Ok(None);
    }

    let retelling = retellings[number];

    Ok(Some(RetellingJson {
        img_src: retelling.0.into(),
        audio_src: retelling.1.into(),
    }))
}

