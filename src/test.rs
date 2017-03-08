use *;
use ganbare::test::*;
use ganbare::quiz::*;
use ganbare::models::*;

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

// Words with only one sentence (and therefore suitable for testing)
// かき
// かく
// いし
// あし

// Pre-test structure
// 4 words: 2 q + 2 e
// 2 sentences 1q + 1e

// Post-test structure
// 4 words: 2 q + 2 e, which of half are new, half are surely-known
// 2 sentences 1q + 1e, which of half are new, half are surely-known


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
                      QuizSerialized::Exercise("あ／し・が生えてる", 7289)];

    let mut quiz = test::get_new_quiz_test(conn, user_id, event, &quizes)?;

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

    let quizes = vec![QuizSerialized::Word("あか・", 1),
                      QuizSerialized::Word("あ・か", 1),
                      QuizSerialized::Question("あか", 1),
                      QuizSerialized::Exercise("あか", 1)];

    let mut quiz = test::get_new_quiz_test(conn, user_id, event, &quizes)?;

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

    let retellings = vec![("static/content_images/retelling/yamada.jpg",
                           "static/content_audio/retelling/yamada.mp3"),
                          ("static/content_images/retelling/nishida.jpg",
                           "static/content_audio/retelling/nishida.mp3"),
                          ("static/content_images/retelling/mari_a.jpg",
                           "static/content_audio/retelling/mari_a.mp3"),
                          ("static/content_images/retelling/mari_b.jpg",
                           "static/content_audio/retelling/mari_b.mp3"),
                          ("static/content_images/retelling/mari_c.jpg",
                           "static/content_audio/retelling/mari_c.mp3"),
                          ("static/content_images/retelling/mari_d.jpg",
                           "static/content_audio/retelling/mari_d.mp3")];
    test::get_new_retelling(conn, user_id, event, &retellings)
}
