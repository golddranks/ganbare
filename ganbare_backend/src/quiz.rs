use super::*;
use rand;
use diesel::expression::dsl::{all, any};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug)]
pub enum Answered {
    W(WAnsweredData),
    Q(QAnsweredData),
    E(EAnsweredData),
}

pub enum QuizType {
    Question(i32),
    Exercise(i32),
    Word(i32),
}

#[derive(Debug)]
pub enum Quiz {
    W(WordJson),
    E(ExerciseJson),
    Q(QuestionJson),
    F(FutureQuiz),
}

#[derive(RustcEncodable, Debug)]
pub struct FutureQuiz {
    quiz_type: &'static str,
    due_date: String,
}

#[derive(RustcEncodable, Debug)]
pub struct QuestionJson {
    quiz_type: &'static str,
    asked_id: i32,
    explanation: String,
    question: String,
    right_a: i32,
    answers: Vec<(i32, String)>,
}

#[derive(RustcEncodable, Debug)]
pub struct ExerciseJson {
    quiz_type: &'static str,
    asked_id: i32,
    word: String,
    explanation: String,
}

#[derive(RustcEncodable, Debug)]
pub struct WordJson {
    quiz_type: &'static str,
    asked_id: i32,
    word: String,
    explanation: String,
    show_accents: bool,
}


fn new_pending_item(conn: &PgConnection, user_id: i32, quiz_n_audio: QuizType) -> Result<PendingItem> {
    use schema::pending_items;
    use self::QuizType::*;

    let (item_type, audio_file_id) = match quiz_n_audio {
        Question(id) => ("question", id),
        Exercise(id) => ("exercise", id),
        Word(id) => ("word", id),
    };

    Ok(diesel::insert(&NewPendingItem{user_id, audio_file_id, item_type})
        .into(pending_items::table)
        .get_result(conn)?)
}

fn register_future_q_answer(conn: &PgConnection, data: &QAskedData) -> Result<()> {
    use schema::q_asked_data;

    diesel::insert(data)
        .into(q_asked_data::table)
        .execute(conn)?;
    Ok(())
}

fn register_future_e_answer(conn: &PgConnection, data: &EAskedData) -> Result<()> {
    use schema::e_asked_data;

    diesel::insert(data)
        .into(e_asked_data::table)
        .execute(conn)?;
    Ok(())
}

fn register_future_w_answer(conn: &PgConnection, data: &WAskedData) -> Result<()> {
    use schema::w_asked_data;

    diesel::insert(data)
        .into(w_asked_data::table)
        .execute(conn)?;
    Ok(())
}

fn log_answer_word(conn : &PgConnection, user : &User, answered: &WAnsweredData) -> Result<()> {
    use schema::{pending_items, w_asked_data, w_answered_data, user_metrics, words};

    let (mut pending_item, asked): (PendingItem, WAskedData) = pending_items::table
        .inner_join(w_asked_data::table)
        .filter(pending_items::id.eq(answered.id))
        .get_result(conn)?;

    // This Q&A is now considered done
    pending_item.pending = false;
    let _ : PendingItem = pending_item.save_changes(conn)?;

    diesel::insert(answered)
        .into(w_answered_data::table)
        .execute(conn)?;

    let word: Word = words::table
        .filter(words::id.eq(asked.word_id))
        .get_result(conn)?;

    let mut metrics : UserMetrics = user_metrics::table
        .filter(user_metrics::id.eq(user.id))
        .get_result(&*conn)?;

    metrics.new_words_today += 1;
    metrics.new_words_since_break += 1;
    let _ : UserMetrics = metrics.save_changes(&*conn)?;

    skill::log_by_id(conn, user, word.skill_nugget, 1)?;

    Ok(())
}

fn log_answer_question(conn : &PgConnection, user : &User, answered: &QAnsweredData) -> Result<(QAskedData, QuestionData, DueItem)> {
    use schema::{pending_items, q_asked_data, q_answered_data, due_items, question_data, quiz_questions};
    use std::cmp::max;

    let (mut pending_item, asked): (PendingItem, QAskedData) = pending_items::table
        .inner_join(q_asked_data::table)
        .filter(pending_items::id.eq(answered.id))
        .get_result(conn)?;

    let correct = asked.correct_qa_id == answered.answered_qa_id.unwrap_or(-1);


    // This Q&A is now considered done
    pending_item.pending = false;
    let _ : PendingItem = pending_item.save_changes(conn)?;

    diesel::insert(answered)
        .into(q_answered_data::table)
        .execute(conn)?;


    // If the answer was wrong, register a new pending question with the same specs right away for a follow-up review
    if !correct {

        let pending_item = new_pending_item(conn, user.id, QuizType::Question(pending_item.audio_file_id))?;
        let asked_data = QAskedData {
            id: pending_item.id,
            question_id: asked.question_id,
            correct_qa_id: asked.correct_qa_id,
        };
        register_future_q_answer(conn, &asked_data)?;

    }


    // Update the data for the question (Diesel doesn't support UPSERT so we have to branch)

    let question: QuizQuestion = quiz_questions::table
        .filter(quiz_questions::id.eq(asked.question_id))
        .get_result(conn)?;

    let questiondata : Option<(QuestionData, DueItem)> = question_data::table
                                        .inner_join(due_items::table)
                                        .filter(due_items::user_id.eq(user.id))
                                        .filter(question_data::question_id.eq(asked.question_id))
                                        .get_result(&*conn)
                                        .optional()?;

    // Update the data for this question (due date, statistics etc.)
    Ok(if let Some((questiondata, mut due_item)) = questiondata {

        let due_delay = if correct { max(due_item.due_delay * 2, 15) } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);
        let streak = if correct {due_item.correct_streak + 1} else { 0 };
        if streak > 2 { skill::log_by_id(conn, user, question.skill_id, 1)?; };

        due_item.due_date = next_due_date;
        due_item.due_delay = due_delay;
        due_item.correct_streak = streak;
        let due_item = due_item.save_changes(conn)?;
        (asked, questiondata, due_item)

    } else { // New!

        let due_delay = if correct { 30 } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);
        skill::log_by_id(conn, user, question.skill_id, 1)?; // First time bonus!

        let due_item = NewDueItem {
            user_id: user.id,
            correct_streak: if correct { 1 } else { 0 },
            due_date: next_due_date,
            due_delay: due_delay,
            item_type: "question".into(),
        };
        let due_item: DueItem = diesel::insert(&due_item)
            .into(due_items::table)
            .get_result(conn)?;
        let questiondata = QuestionData {
            question_id: asked.question_id,
            due: due_item.id,
        };
        let questiondata = diesel::insert(&questiondata)
            .into(question_data::table)
            .get_result(conn)?;
        (asked, questiondata, due_item)
    })
}

fn log_answer_exercise(conn: &PgConnection, user: &User, answered: &EAnsweredData) -> Result<(ExerciseData, DueItem)> {
    use schema::{pending_items, e_asked_data, e_answered_data, due_items, exercise_data, words};
    use std::cmp::max;

    let correct = answered.answer_level > 0;

    let (mut pending_item, asked): (PendingItem, EAskedData) = pending_items::table
        .inner_join(e_asked_data::table)
        .filter(pending_items::id.eq(answered.id))
        .get_result(conn)?;

    // This Q&A is now considered done
    pending_item.pending = false;
    let _ : PendingItem = pending_item.save_changes(conn)?;

    diesel::insert(answered)
        .into(e_answered_data::table)
        .execute(conn)?;

    let w : Word = words::table
                    .filter(words::id.eq(asked.word_id))
                    .get_result(conn)?;

    let exercisedata : Option<(ExerciseData, DueItem)> = exercise_data::table
                                        .inner_join(due_items::table)
                                        .filter(due_items::user_id.eq(user.id))
                                        .filter(exercise_data::word_id.eq(asked.word_id))
                                        .get_result(&*conn)
                                        .optional()?;

    // Update the data for this word exercise (due date, statistics etc.)
    Ok(if let Some((exercisedata, mut due_item)) = exercisedata {

        let due_delay = if correct { max(due_item.due_delay * 2, 15) } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);
        let streak = if correct {due_item.correct_streak + 1} else { 0 };
        if streak > 2 { skill::log_by_id(conn, user, w.skill_nugget, 1)?; };

        due_item.due_date = next_due_date;
        due_item.due_delay = due_delay;
        due_item.correct_streak = streak;
        let due_item = due_item.save_changes(conn)?;
        (exercisedata, due_item)

    } else { // New!

        let due_delay = if correct { 30 } else { 0 };
        let next_due_date = chrono::UTC::now() + chrono::Duration::seconds(due_delay as i64);
        skill::log_by_id(conn, user, w.skill_nugget, 1)?; // First time bonus!

        let due_item = NewDueItem {
            user_id: user.id,
            correct_streak: if correct { 1 } else { 0 },
            due_date: next_due_date,
            due_delay: due_delay,
            item_type: "exercise".into(),
        };
        let due_item: DueItem = diesel::insert(&due_item)
            .into(due_items::table)
            .get_result(conn)?;
        let exercisedata = ExerciseData {
            due: due_item.id,
            word_id: asked.word_id,
        };
        let exercisedata = diesel::insert(&exercisedata)
            .into(exercise_data::table)
            .get_result(conn)
            .chain_err(|| "Couldn't save the question tally data to database!")?;
        (exercisedata, due_item)
    })
}

pub fn load_question(conn : &PgConnection, id: i32 ) -> Result<Option<(QuizQuestion, Vec<Answer>, Vec<Vec<AudioFile>>)>> {
    use schema::{quiz_questions, question_answers, audio_bundles};

    let qq : Option<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.eq(id))
        .get_result(&*conn)
        .optional()?;

    let qq = try_or!{ qq, else return Ok(None) };

    let (aas, q_bundles) : (Vec<Answer>, Vec<AudioBundle>) = question_answers::table
        .inner_join(audio_bundles::table)
        .filter(question_answers::question_id.eq(qq.id))
        .load(&*conn)?
        .into_iter().unzip();

    let q_audio_files = audio::load_from_bundles(&*conn, &q_bundles)?;
    
    Ok(Some((qq, aas, q_audio_files)))
}

fn load_word(conn : &PgConnection, id: i32 ) -> Result<Option<(Word, Vec<AudioFile>)>> {
    use schema::{words};

    let ww : Option<Word> = words::table
        .filter(words::id.eq(id))
        .get_result(&*conn)
        .optional()?;

    let ww = try_or!{ ww, else return Ok(None) };

    let w_audio_files = audio::load_from_bundle(conn, ww.audio_bundle)?;
    
    Ok(Some((ww, w_audio_files)))
}

fn get_pending_item(conn: &PgConnection, user_id: i32) -> Result<Option<(PendingItem, Quiz)>> {
    use schema::{pending_items, q_asked_data, e_asked_data, w_asked_data};

    let pending_item: Option<PendingItem> = pending_items::table
        .filter(pending_items::user_id.eq(user_id))
        .filter(pending_items::pending.eq(true))
        .get_result(conn)
        .optional()?;

    let quiz_type = match pending_item {
        Some(ref pi) if pi.item_type == "question" => {

            let asked: QAskedData = q_asked_data::table
                .filter(q_asked_data::id.eq(pi.id))
                .get_result(conn)?;

            let (question, answers, _) = try_or!{ load_question(conn, asked.question_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::Q(QuestionJson {
                quiz_type: "question",
                asked_id: pi.id,
                explanation: question.q_explanation,
                question: question.question_text,
                right_a: asked.correct_qa_id,
                answers: answers.into_iter().map(|a| (a.id, a.answer_text)).collect(),
            })

        },
        Some(ref pi) if pi.item_type == "exercise" => {

            let asked: EAskedData = e_asked_data::table
                .filter(e_asked_data::id.eq(pi.id))
                .get_result(conn)?;

            let (word, _) = try_or!{ load_word(conn, asked.word_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::E(ExerciseJson {
                quiz_type: "exercise",
                asked_id: pi.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
            })

        },
        Some(ref pi) if pi.item_type == "word" => {

            let asked: WAskedData = w_asked_data::table
                .filter(w_asked_data::id.eq(pi.id))
                .get_result(conn)?;

            let (word, _) = try_or!{ load_word(conn, asked.word_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::W(WordJson {
                quiz_type: "word",
                asked_id: pi.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
                show_accents: asked.show_accents,
            })
        },
        Some(_) => unreachable!("Bug: There is only three kinds of quiz types!"),
        None => return Ok(None),
    };

    Ok(Some((pending_item.expect("Didn't match None just a moment ago."), quiz_type)))
}

fn get_due_items(conn : &PgConnection, user_id: i32, allow_peeking: bool) -> Result<Vec<(DueItem, QuizType)>> {
    use schema::{due_items, question_data, exercise_data};

    let due_questions: Vec<(DueItem, Option<QuestionData>)>;
    let due_exercises: Vec<(DueItem, Option<ExerciseData>)>;
    if allow_peeking { 

        due_questions = due_items::table
            .left_outer_join(question_data::table)
            .filter(due_items::user_id.eq(user_id))
            .order(due_items::due_date.asc())
            .limit(5)
            .get_results(&*conn)?;

        due_exercises = due_items::table
            .left_outer_join(exercise_data::table)
            .filter(due_items::user_id.eq(user_id))
            .order(due_items::due_date.asc())
            .limit(5)
            .get_results(&*conn)?;

    } else {

        due_questions = due_items::table
            .left_outer_join(question_data::table)
            .filter(due_items::user_id.eq(user_id))
            .filter(due_items::due_date.lt(chrono::UTC::now()))
            .order(due_items::due_date.asc())
            .limit(5)
            .get_results(&*conn)?;

        due_exercises = due_items::table
            .left_outer_join(exercise_data::table)
            .filter(due_items::user_id.eq(user_id))
            .filter(due_items::due_date.lt(chrono::UTC::now()))
            .order(due_items::due_date.asc())
            .limit(5)
            .get_results(&*conn)?;

    };

    let due_items = due_questions.into_iter().zip(due_exercises.into_iter()).map(
            |zipped| match zipped {
                ((di, Some(question)), (_, None)) => (di, QuizType::Question(question.question_id)),
                ((_, None), (di, Some(exercise))) => (di, QuizType::Exercise(exercise.word_id)),
                _ => unreachable!(),
            }
            ).collect();

    Ok(due_items)
}

fn get_new_questions(conn : &PgConnection, user_id : i32) -> Result<Vec<QuizQuestion>> {
    use schema::{quiz_questions, question_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(question_data::table)
        .select(question_data::question_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_questions : Vec<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.ne(all(dues)))
        .filter(quiz_questions::skill_id.eq(any(skills)))
        .filter(quiz_questions::published.eq(true))
        .limit(5)
        .order(quiz_questions::id.asc())
        .get_results(conn)?;

    Ok(new_questions)
}

fn get_new_exercises(conn : &PgConnection, user_id : i32) -> Result<Vec<Word>> {
    use schema::{words, exercise_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(exercise_data::table)
        .select(exercise_data::word_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_questions : Vec<Word> = words::table
        .filter(words::id.ne(all(dues)))
        .filter(words::skill_nugget.eq(any(skills)))
        .filter(words::published.eq(true))
        .limit(5)
        .order(words::id.asc())
        .get_results(conn)?;

    Ok(new_questions)
}

fn get_new_words(conn : &PgConnection, user_id : i32) -> Result<Vec<Word>> {
    use diesel::expression::dsl::*;
    use schema::{pending_items, words, w_asked_data};

    let seen = pending_items::table
        .inner_join(w_asked_data::table)
        .select(w_asked_data::word_id)
        .filter(pending_items::user_id.eq(user_id));

    let new_words : Vec<Word> = words::table
        .filter(words::id.ne(all(seen)))
        .filter(words::published.eq(true))
        .limit(5)
        .order(words::id.asc())
        .get_results(conn)
        .chain_err(|| "Can't get new words!")?;

    Ok(new_words)
}

fn ask_new_question(conn: &PgConnection, id: i32) -> Result<(QuizQuestion, i32, Vec<Answer>, i32)> {
    use rand::Rng;

    let (question, answers, q_audio_bundles) = try_or!{ load_question(conn, id)?,
                else return Err(ErrorKind::DatabaseOdd("This function was called on the premise that the data exists!").to_err()) };
    
    let mut rng = rand::thread_rng();
    let random_answer_index = rng.gen_range(0, answers.len());
    let right_answer_id = answers[random_answer_index].id;
    let q_audio_bundle = &q_audio_bundles[random_answer_index];
    let q_audio_file = try_or!{ rng.choose(q_audio_bundle),
                else return Err(ErrorKind::DatabaseOdd("Bug: Audio bundles should always have more than zero members when created.").to_err()) };

    Ok((question, right_answer_id, answers, q_audio_file.id))
}

pub fn get_new_quiz(conn : &PgConnection, user : &User) -> Result<Option<Quiz>> {
    use schema::user_metrics;
    use rand::Rng;

    // Pending item first (items that were asked, but not answered because of loss of connection, user closing the session etc.)

    if let Some((_, quiztype)) = get_pending_item(conn, user.id)? {
        return Ok(Some(quiztype));
    }

    // After that, questions & exercises reviews that are overdue, after that new ones

    let quiz_data =
    if let Some((due, quiztype)) = get_due_items(conn, user.id, false)?.into_iter().next() {
        Some((Some(due), quiztype))

    } else if let Some(q) = get_new_questions(conn, user.id)?.into_iter().next() {
        Some((None, QuizType::Question(q.id)))

    } else if let Some(e) = get_new_exercises(conn, user.id)?.into_iter().next() {
        Some((None, QuizType::Exercise(e.id)))

    } else { None };

    match quiz_data {
        Some((_, QuizType::Question(id))) => {

            let (question, right_a, answers, q_audio_id) = ask_new_question(conn, id)?;

            let pending_item = new_pending_item(conn, user.id, QuizType::Question(q_audio_id))?;

            let asked_data = QAskedData {
                id: pending_item.id,
                question_id: question.id,
                correct_qa_id: right_a,
            };

            register_future_q_answer(conn, &asked_data)?;

            let quiz_json = QuestionJson {
                quiz_type: "question",
                asked_id: pending_item.id,
                explanation: question.q_explanation,
                question: question.question_text,
                right_a: right_a,
                answers: answers.into_iter().map(|a| (a.id, a.answer_text)).collect(),
            };

            return Ok(Some(Quiz::Q(quiz_json)))
    
        },
        Some((_, QuizType::Exercise(id))) => {

            let mut rng = rand::thread_rng();

            let (word, audio_files) = try_or!( load_word(conn, id)?, else return Ok(None));
            let audio_file = try_or!{ rng.choose(&audio_files),
                else return Err(ErrorKind::DatabaseOdd("Bug: Audio bundles should always have more than zero members when created.").to_err()) };

            let pending_item = new_pending_item(conn, user.id, QuizType::Exercise(audio_file.id))?;

            let asked_data = EAskedData {
                id: pending_item.id,
                word_id: word.id,
            };

            register_future_e_answer(conn, &asked_data)?;

            let quiz_json = ExerciseJson{
                quiz_type: "exercise",
                asked_id: pending_item.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
            };

            return Ok(Some(Quiz::E(quiz_json)))
        },
        _ => (),
    };

    // No questions & exercises available ATM, checking words

    let metrics : UserMetrics = user_metrics::table.filter(user_metrics::id.eq(user.id)).get_result(&*conn)?;
    
    if metrics.new_words_today <= 18 || metrics.new_words_since_break <= 6 {
        let mut words = get_new_words(&*conn, user.id)?;
        if words.len() > 0 {
            let mut rng = rand::thread_rng();

            let the_word = words.swap_remove(0);
            let audio_files = audio::load_from_bundle(&*conn, the_word.audio_bundle)?;
            let audio_file = try_or!{ rng.choose(&audio_files),
                else return Err(ErrorKind::DatabaseOdd("Bug: Audio bundles should always have more than zero members when created.").to_err()) };
            let show_accents = user::check_user_group(conn, user, "output_group")?;

            let pending_item = new_pending_item(conn, user.id, QuizType::Word(audio_file.id))?;

            let asked_data = WAskedData {
                id: pending_item.id,
                word_id: the_word.id,
                show_accents,
            };

            register_future_w_answer(conn, &asked_data)?;

            let quiz_json = WordJson{
                quiz_type: "word",
                word: the_word.word.nfc().collect::<String>(),
                explanation: the_word.explanation,
                asked_id: pending_item.id,
                show_accents,
            };

            return Ok(Some(Quiz::W(quiz_json)));
        }
    }

    // Peeking for the future

    if let Some((due, _)) = get_due_items(conn, user.id, true)?.into_iter().next() {

        let due_date = due.due_date.to_rfc3339();

        return Ok(Some(Quiz::F(FutureQuiz{ quiz_type: "future", due_date })));

    } 
    Ok(None)
}


pub fn get_next_quiz(conn : &PgConnection, user : &User, answer_enum: Answered)
    -> Result<Option<Quiz>>
{
    match answer_enum {
        Answered::W(answer_word) => {
            log_answer_word(conn, user, &answer_word)?;
        },
        Answered::E(exercise) => {
            log_answer_exercise(conn, user, &exercise)?;
        },
        Answered::Q(answer) => {
            log_answer_question(conn, user, &answer)?;
        },
    }
    get_new_quiz(conn, user)
}


