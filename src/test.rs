use std::fs;

use crate::*;
use ganbare::test::*;
use ganbare::quiz::*;
use ganbare::models::*;
use ganbare_backend::ResultExt;

pub fn get_next_quiz_pretest(conn: &Connection,
                             user_id: i32,
                             answer_enum: Answered,
                             event: &Event)
                             -> Result<Option<Quiz>> {
    test::save_answer_test_item(conn, user_id, event, &answer_enum)?;
    test::get_new_quiz_pretest(conn, user_id, event)
}


pub fn get_next_quiz_posttest(conn: &Connection,
                              user_id: i32,
                              answer_enum: Answered,
                              event: &Event)
                              -> Result<Option<Quiz>> {
    test::save_answer_test_item(conn, user_id, event, &answer_enum)?;
    test::get_new_quiz_posttest(conn, user_id, event)
}

pub fn get_next_retelling_posttest(conn: &Connection,
                                   user_id: i32,
                                   event: &Event)
                                   -> Result<Option<RetellingJson>> {

    test::save_retelling(conn, user_id, event)?;
    test::get_new_retelling_posttest(conn, user_id, event)
}

pub fn get_next_retelling_pretest(conn: &Connection,
                                  user_id: i32,
                                  event: &Event)
                                  -> Result<Option<RetellingJson>> {

    test::save_retelling(conn, user_id, event)?;
    test::get_new_retelling_pretest(conn, user_id, event)
}

pub fn get_new_quiz_pretest(conn: &Connection,
                            user_id: i32,
                            event: &Event)
                            -> Result<Option<Quiz>> {
    let tsv = fs::read_to_string("private_assets/pretest.tsv")
        .or_else(|_| fs::read_to_string("private_assets/test_placeholder.tsv"))
        .chain_err(|| "No pretest.tsv or even placefolder found!")?;
    let quizes = ganbare::quiz::read_quiz_tsv(tsv)
        .chain_err(|| "Malformed pretest quiz .tsv")?;
    let mut quiz = test::get_new_quiz_test(conn, user_id, event, &quizes)
        .chain_err(|| "Failed getting new pretest quiz item.")?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = "pretest";
    }

    Ok(quiz)
}

pub fn get_new_quiz_posttest(conn: &Connection,
                             user_id: i32,
                             event: &Event)
                             -> Result<Option<Quiz>> {
    let tsv = fs::read_to_string("private_assets/posttest.tsv")
        .or_else(|_| fs::read_to_string("private_assets/test_placeholder.tsv"))
        .chain_err(|| "No posttest.tsv or even placefolder found!")?;
    let quizes = ganbare::quiz::read_quiz_tsv(tsv)
        .chain_err(|| "Malformed posttest quiz .tsv")?;
    let mut quiz = test::get_new_quiz_test(conn, user_id, event, &quizes)
        .chain_err(|| "Failed getting new posttest quiz item.")?;

    if let Some(Quiz::E(ref mut e)) = quiz {
        e.must_record = true;
        e.event_name = "posttest";
    }

    Ok(quiz)
}

pub fn get_new_retelling_pretest(conn: &Connection,
                                 user_id: i32,
                                 event: &Event)
                                 -> Result<Option<RetellingJson>> {

    let retellings = vec![("static/content_images/retelling/yamada.png",
                           "static/content_audio/retelling/yamada.mp3"),
                          ("static/content_images/retelling/nishida.png",
                           "static/content_audio/retelling/nishida.mp3"),
                          ("static/content_images/retelling/mari_a.png",
                           "static/content_audio/retelling/mari_a.mp3"),
                          ("static/content_images/retelling/mari_b.png",
                           "static/content_audio/retelling/mari_b.mp3"),
                          ("static/content_images/retelling/mari_c.png",
                           "static/content_audio/retelling/mari_c.mp3"),
                          ("static/content_images/retelling/mari_d.png",
                           "static/content_audio/retelling/mari_d.mp3")];
    test::get_new_retelling(conn, user_id, event, &retellings)
}

pub fn get_new_retelling_posttest(conn: &Connection,
                                  user_id: i32,
                                  event: &Event)
                                  -> Result<Option<RetellingJson>> {

    let retellings = vec![("static/content_images/retelling/yamada.png",
                           "static/content_audio/retelling/yamada.mp3"),
                          ("static/content_images/retelling/nishida.png",
                           "static/content_audio/retelling/nishida.mp3"),
                          ("static/content_images/retelling/mari_a.png",
                           "static/content_audio/retelling/mari_a.mp3"),
                          ("static/content_images/retelling/mari_b.png",
                           "static/content_audio/retelling/mari_b.mp3"),
                          ("static/content_images/retelling/mari_c.png",
                           "static/content_audio/retelling/mari_c.mp3"),
                          ("static/content_images/retelling/mari_d.png",
                           "static/content_audio/retelling/mari_d.mp3")];
    test::get_new_retelling(conn, user_id, event, &retellings)
}
