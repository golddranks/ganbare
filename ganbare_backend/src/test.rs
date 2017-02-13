use super::*;
use quiz::{Answered, Quiz, QuizSerialized};
use rustc_serialize::json;


fn save_answer_test_item(conn: &Connection,
                         user: &User,
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

    event::save_userdata(conn, event, user, Some("pending_test_item"), "0")?;

    let mut number = event::get_userdata(conn, event, user, "quiz_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);
    let answer_key = "answer_".to_string() + &number.to_string();
    let answer_json = json::encode(&answer_enum).unwrap();
    event::save_userdata(conn,
                         event,
                         user,
                         Some(answer_key.as_str()),
                         answer_json.as_str())?;
    number += 1;
    event::save_userdata(conn, event, user, Some("quiz_number"), &number.to_string())?;

    debug!("Pending test item unpended & answer saved.");

    Ok(())
}

fn get_new_quiz_test(conn: &Connection,
                     user: &User,
                     event: &Event,
                     quizes: &[QuizSerialized])
                     -> Result<Option<Quiz>> {

    let number = event::get_userdata(conn, event, user, "quiz_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);
    let pending_test_item = event::get_userdata(conn, event, user, "pending_test_item")
        ?
        .and_then(|d| d.data.parse::<i32>().ok())
        .unwrap_or(0);

    let quiz = if pending_test_item == 0 {
        // We can show a new item, since to old one was answered to

        if number == quizes.len() {
            event::set_done(conn, &event.name, user)?;
            return Ok(None);
        }

        let (quiz, pi_id) = quiz::test_item(conn, user, &quizes[number])?;
        event::save_userdata(conn,
                             event,
                             user,
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

pub fn get_next_quiz_pretest(conn: &Connection,
                             user: &User,
                             answer_enum: Answered,
                             event: &Event)
                             -> Result<Option<Quiz>> {
    save_answer_test_item(conn, user, event, &answer_enum)?;
    get_new_quiz_pretest(conn, user, event)
}


pub fn get_next_quiz_posttest(conn: &Connection,
                              user: &User,
                              answer_enum: Answered,
                              event: &Event)
                              -> Result<Option<Quiz>> {
    save_answer_test_item(conn, user, event, &answer_enum)?;
    get_new_quiz_posttest(conn, user, event)
}

#[derive(RustcEncodable)]
pub struct RetellingJson {
    img_src: String,
    audio_src: String,
}

fn get_new_retelling(conn: &Connection,
                     user: &User,
                     event: &Event,
                     retellings: &[(&'static str, &'static str)])
                     -> Result<Option<RetellingJson>> {

    let number = event::get_userdata(conn, event, user, "retelling_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);

    if number == retellings.len() {
        event::set_done(conn, &event.name, user)?;
        return Ok(None);
    }

    let retelling = retellings[number];

    Ok(Some(RetellingJson {
        img_src: retelling.0.into(),
        audio_src: retelling.1.into(),
    }))
}

pub fn get_next_retelling_posttest(conn: &Connection,
                                   user: &User,
                                   event: &Event)
                                   -> Result<Option<RetellingJson>> {

    let number = event::get_userdata(conn, event, user, "retelling_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0) + 1;
    event::save_userdata(conn,
                         event,
                         user,
                         Some("retelling_number"),
                         &number.to_string())?;
    get_new_retelling_posttest(conn, user, event)
}

pub fn get_next_retelling_pretest(conn: &Connection,
                                  user: &User,
                                  event: &Event)
                                  -> Result<Option<RetellingJson>> {

    let number = event::get_userdata(conn, event, user, "retelling_number")
        ?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0) + 1;
    event::save_userdata(conn,
                         event,
                         user,
                         Some("retelling_number"),
                         &number.to_string())?;
    get_new_retelling_pretest(conn, user, event)
}

pub fn get_new_quiz_pretest(conn: &Connection,
                            user: &User,
                            event: &Event)
                            -> Result<Option<Quiz>> {

    let quizes = vec![QuizSerialized::Word("いし・", 5355),
                      QuizSerialized::Word("い・し", 5367),
                      QuizSerialized::Question("い・し", 6827),
                      QuizSerialized::Word("かき", 5112),
                      QuizSerialized::Word("か・き", 5129),
                      QuizSerialized::Exercise("か・き", 3675),
                      QuizSerialized::Word("か・く", 3966),
                      QuizSerialized::Word("はる", 1345),
                      QuizSerialized::Word("かく", 3659),
                      QuizSerialized::Word("は・る", 1356),
                      QuizSerialized::Question("か・く", 5420),
                      QuizSerialized::Exercise("は・る", 5287),
                      QuizSerialized::Word("／あ・きがきた", 5061),
                      QuizSerialized::Word("あ／き・がきた", 4759),
                      QuizSerialized::Question("あ／き・がきた", 5045),
                      QuizSerialized::Word("／あ・しが生えてる", 7290),
                      QuizSerialized::Word("あ／し・が生えてる", 7289),
                      QuizSerialized::Exercise("あ／し・が生えてる", 7289),
                    ];

    let mut quiz = get_new_quiz_test(conn, user, event, &quizes)?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = "pretest";
    }

    Ok(quiz)
}

pub fn get_new_quiz_posttest(conn: &Connection,
                             user: &User,
                             event: &Event)
                             -> Result<Option<Quiz>> {

    let quizes = vec![QuizSerialized::Word("あか・", 1),
                      QuizSerialized::Word("あ・か", 1),
                      QuizSerialized::Question("あか", 1),
                      QuizSerialized::Exercise("あか", 1)];

    let mut quiz = get_new_quiz_test(conn, user, event, &quizes)?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = "posttest";
    }

    Ok(quiz)
}

pub fn get_new_retelling_pretest(conn: &Connection,
                                 user: &User,
                                 event: &Event)
                                 -> Result<Option<RetellingJson>> {

    let retellings =
        vec![("static/content_images/retelling/yamada.png", "static/content_audio/retelling/yamada.mp3"),
             ("static/content_images/retelling/nishida.png", "static/content_audio/retelling/nishida.mp3"),
             ("static/content_images/retelling/mari_a.png", "static/content_audio/retelling/mari_a.mp3"),
             ("static/content_images/retelling/mari_b.png", "static/content_audio/retelling/mari_b.mp3"),
             ("static/content_images/retelling/mari_c.png", "static/content_audio/retelling/mari_c.mp3"),
             ("static/content_images/retelling/mari_d.png", "static/content_audio/retelling/mari_d.mp3")];
    get_new_retelling(conn, user, event, &retellings)
}

pub fn get_new_retelling_posttest(conn: &Connection,
                                  user: &User,
                                  event: &Event)
                                  -> Result<Option<RetellingJson>> {

    let retellings =
        vec![("static/content_images/retelling/yamada.jpg", "static/content_audio/retelling/yamada.mp3"),
             ("static/content_images/retelling/nishida.jpg", "static/content_audio/retelling/nishida.mp3"),
             ("static/content_images/retelling/mari_a.jpg", "static/content_audio/retelling/mari_a.mp3"),
             ("static/content_images/retelling/mari_b.jpg", "static/content_audio/retelling/mari_b.mp3"),
             ("static/content_images/retelling/mari_c.jpg", "static/content_audio/retelling/mari_c.mp3"),
             ("static/content_images/retelling/mari_d.jpg", "static/content_audio/retelling/mari_d.mp3")];
    get_new_retelling(conn, user, event, &retellings)
}
