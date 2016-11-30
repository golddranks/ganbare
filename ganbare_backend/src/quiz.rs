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

#[derive(Debug)]
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












/* ANSWERING */


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

fn log_answer_due_item(conn: &PgConnection, mut due_item: DueItem, skill_id: i32, correct: bool, metrics: &UserMetrics) -> Result<DueItem> {
    use std::cmp::max;

    due_item.correct_streak_this_time = if correct {due_item.correct_streak_this_time + 1} else { 0 };
    due_item.cooldown_delay = chrono::UTC::now() + chrono::Duration::seconds(metrics.cooldown_delay as i64);

    if due_item.correct_streak_this_time >= metrics.streak_limit {
        due_item.correct_streak_this_time = 0;
        due_item.correct_streak_overall = if correct {due_item.correct_streak_overall + 1} else { 0 };
        due_item.due_delay = if correct { max(due_item.due_delay * metrics.delay_multiplier, metrics.initial_delay) } else { 0 };
        due_item.due_date = chrono::UTC::now() + chrono::Duration::seconds(due_item.due_delay as i64);
        if due_item.correct_streak_overall > 3 {
            skill::log_by_id(conn, due_item.user_id, skill_id, 1)?;
        };
    }

    Ok(due_item.save_changes(conn)?)
}

fn log_answer_new_due_item(conn: &PgConnection, user_id: i32, item_type: &str, skill_id: i32, correct: bool, metrics: &UserMetrics) -> Result<DueItem> {
    use schema::due_items;

    // Diesel doesn't have UPSERT so we have to initialize separately.

    skill::log_by_id(conn, user_id, skill_id, 1)?; // First time bonus!

    let new_due_item = NewDueItem {
        user_id: user_id,
        correct_streak_this_time: 0,
        correct_streak_overall: 0,
        due_date: chrono::UTC::now(),
        due_delay: 0,
        cooldown_delay: chrono::UTC::now(),
        item_type: item_type,
    };

    let due_item : DueItem = diesel::insert(&new_due_item)
        .into(due_items::table)
        .get_result(conn)?;

    Ok(log_answer_due_item(conn, due_item, skill_id, correct, metrics)?)
}

fn log_answer_word(conn : &PgConnection, user : &User, answered: &WAnsweredData) -> Result<()> {
    use schema::{pending_items, w_asked_data, w_answered_data, words};

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

    skill::log_by_id(conn, user.id, word.skill_nugget, 1)?;

    Ok(())
}

fn log_answer_question(conn : &PgConnection, user : &User, answered: &QAnsweredData, metrics: &UserMetrics) -> Result<(QAskedData, QuestionData, DueItem)> {
    use schema::{pending_items, q_asked_data, q_answered_data, due_items, question_data, quiz_questions};

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
    Ok(if let Some((questiondata, due_item)) = questiondata {

        let due_item = log_answer_due_item(conn, due_item, question.skill_id, correct, metrics)?;

        (asked, questiondata, due_item)

    } else { // New!

        let due_item = log_answer_new_due_item(conn, user.id, "question", question.skill_id, correct, metrics)?;

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

fn log_answer_exercise(conn: &PgConnection, user: &User, answered: &EAnsweredData, metrics: &UserMetrics) -> Result<(ExerciseData, DueItem)> {
    use schema::{pending_items, e_asked_data, e_answered_data, due_items, exercise_data, words};

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
    Ok(if let Some((exercisedata, due_item)) = exercisedata {

        let due_item = log_answer_due_item(conn, due_item, w.skill_nugget, correct, metrics)?;

        (exercisedata, due_item)

    } else { // New!

        let due_item = log_answer_new_due_item(conn, user.id, "exercise", w.skill_nugget, correct, metrics)?;

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










/* FETCHING & CHOOSING QUESTIONS */


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

    let q_audio_files = audio::load_all_from_bundles(&*conn, &q_bundles)?;
    
    Ok(Some((qq, aas, q_audio_files)))
}

fn load_word(conn : &PgConnection, id: i32 ) -> Result<Option<Word>> {
    use schema::{words};

    Ok(words::table
        .filter(words::id.eq(id))
        .get_result(&*conn)
        .optional()?)
}

fn get_pending_item(conn: &PgConnection, user_id: i32) -> Result<Option<Quiz>> {
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

            let word = try_or!{ load_word(conn, asked.word_id)?,
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

            let word = try_or!{ load_word(conn, asked.word_id)?,
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
        None => {
            return Ok(None)
        },
    };

    Ok(Some(quiz_type))
}

fn get_next_due_item(conn : &PgConnection, user_id: i32) -> Result<Option<(DueItem, QuizType)>> {
    use schema::{due_items, question_data, exercise_data};

    let due_questions: Option<(DueItem, Option<QuestionData>)> = due_items::table
        .left_outer_join(question_data::table)
        .filter(due_items::user_id.eq(user_id))
        .order(due_items::due_date.asc())
        .first(conn)
        .optional()?;

    let due_exercises: Option<(DueItem, Option<ExerciseData>)> = due_items::table
        .left_outer_join(exercise_data::table)
        .filter(due_items::user_id.eq(user_id))
        .order(due_items::due_date.asc())
        .first(conn)
        .optional()?;

    let due_item = due_questions.into_iter().zip(due_exercises).next().map(
            |zipped| match zipped {
                ((di, Some(question)), (_, None)) => (di, QuizType::Question(question.question_id)),
                ((_, None), (di, Some(exercise))) => (di, QuizType::Exercise(exercise.word_id)),
                _ => unreachable!(),
            });

    Ok(due_item)
}

fn get_random_overdue_item(conn : &PgConnection, user_id: i32) -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due_questions: Option<(DueItem, Option<QuestionData>)> = due_items::table
        .left_outer_join(question_data::table)
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .filter(due_items::cooldown_delay.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;

    let due_exercises: Option<(DueItem, Option<ExerciseData>)> = due_items::table
        .left_outer_join(exercise_data::table)
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .filter(due_items::cooldown_delay.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;


    let due_item = due_questions.into_iter().zip(due_exercises).next().map(
            |zipped| match zipped {
                ((_, Some(question)), (_, None)) => QuizType::Question(question.question_id),
                ((_, None), (_, Some(exercise))) => QuizType::Exercise(exercise.word_id),
                _ => unreachable!(),
            });

    Ok(due_item)
}

fn get_random_overdue_cooldown_item(conn : &PgConnection, user_id: i32) -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due_questions: Option<(DueItem, Option<QuestionData>)> = due_items::table
        .left_outer_join(question_data::table)
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;

    let due_exercises: Option<(DueItem, Option<ExerciseData>)> = due_items::table
        .left_outer_join(exercise_data::table)
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;


    let due_item = due_questions.into_iter().zip(due_exercises).next().map(
            |zipped| match zipped {
                ((_, Some(question)), (_, None)) => QuizType::Question(question.question_id),
                ((_, None), (_, Some(exercise))) => QuizType::Exercise(exercise.word_id),
                _ => unreachable!(),
            });

    Ok(due_item)
}

fn get_new_question(conn : &PgConnection, user_id : i32) -> Result<Option<QuizQuestion>> {
    use schema::{quiz_questions, question_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(question_data::table)
        .select(question_data::question_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_question : Option<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.ne(all(dues)))
        .filter(quiz_questions::skill_id.eq(any(skills)))
        .filter(quiz_questions::published.eq(true))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(new_question)
}

fn get_new_exercise(conn : &PgConnection, user_id : i32) -> Result<Option<Word>> {
    use schema::{words, exercise_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(exercise_data::table)
        .select(exercise_data::word_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_exercise : Option<Word> = words::table
        .filter(words::id.ne(all(dues)))
        .filter(words::skill_nugget.eq(any(skills)))
        .filter(words::published.eq(true))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(new_exercise)
}

fn get_new_random_word(conn : &PgConnection, user_id : i32) -> Result<Option<Word>> {
    use diesel::expression::dsl::*;
    use schema::{pending_items, words, w_asked_data};

    let seen = pending_items::table
        .inner_join(w_asked_data::table)
        .select(w_asked_data::word_id)
        .filter(pending_items::user_id.eq(user_id));

    let new_word : Option<Word> = words::table
        .filter(words::id.ne(all(seen)))
        .filter(words::published.eq(true))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(new_word)
}

fn get_new_paired_word(conn : &PgConnection, user_id : i32) -> Result<Option<Word>> {
    use diesel::expression::dsl::*;
    use schema::{pending_items, words, w_asked_data, skill_nuggets, skill_data};

    let seen = pending_items::table
        .inner_join(w_asked_data::table)
        .select(w_asked_data::word_id)
        .filter(pending_items::user_id.eq(user_id));

    let other_pair_seen = skill_nuggets::table
        .inner_join(skill_data::table)
        .select(skill_nuggets::id)
        .filter(skill_data::user_id.eq(user_id))
        .filter(skill_data::skill_level.gt(0));

    let new_word : Option<Word> = words::table
        .filter(words::id.ne(all(seen)))
        .filter(words::published.eq(true))
        .filter(words::skill_nugget.eq(any(other_pair_seen)))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(new_word)
}

fn introduce_new_word(conn : &PgConnection, metrics: &mut UserMetrics) -> Result<Option<Word>> {


    if metrics.new_words_since_break >= metrics.max_words_since_break
    || metrics.new_words_today >= metrics.max_words_today {
        return Ok(None)
    }

    let user_id = metrics.id;

    let paired_word = get_new_paired_word(conn, user_id)?;
    
    let word = match paired_word {
        None => get_new_random_word(conn, user_id)?,
        some => some,
    };

    Ok(word)
}


fn check_break(conn: &PgConnection, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {
    use std::cmp::{max};
    use chrono::Duration;

    let user_id = metrics.id;

    // We are having 1 AM as the change point of the day
    // (That's 3 AM in Finland, where the users of this app are likely to reside)
    if chrono::UTC::now() > (metrics.today.date().and_hms(1, 0, 0) + Duration::hours(24)) {
        metrics.today = chrono::UTC::today().and_hms(1, 0, 0);
        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
    }

    // Start a break:

    if metrics.new_words_since_break >= metrics.max_words_since_break
        && metrics.quizes_since_break >= metrics.max_quizes_since_break
    {
        let time_since_last_break = chrono::UTC::now() - metrics.break_until;

        let discounted_breaktime = max(Duration::seconds(0),
                Duration::seconds(metrics.break_length as i64) - time_since_last_break);

        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.break_until = chrono::UTC::now()
                                + discounted_breaktime;
    }

    if metrics.new_words_today >= metrics.max_words_today
        && metrics.quizes_today >= metrics.max_quizes_today
    {
        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
        metrics.break_until = metrics.today + Duration::hours(24);
    }


    if chrono::UTC::now() > metrics.break_until { return Ok(None) } // Not on break

    // Peeking for the future

    if let Some(_) = get_new_random_word(conn, user_id)? { // There's still new words to learn

        let due_date = metrics.break_until;

        let due_string = due_date.to_rfc3339();
        return Ok(Some(Quiz::F(FutureQuiz{ quiz_type: "future", due_date: due_string })));
    }

    if let Some((due, _)) = get_next_due_item(conn, user_id)? { // Only quizes left, so their due dates affect the next due.

        let due_date = max(due.due_date, metrics.break_until);

        let due_string = due_date.to_rfc3339();
        return Ok(Some(Quiz::F(FutureQuiz{ quiz_type: "future", due_date: due_string })));

    } 
    Ok(None)
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

fn get_cooldown_q_or_e(conn: &PgConnection, user: &User, metrics: &UserMetrics) -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break
    || metrics.quizes_today >= metrics.max_quizes_today {
        return Ok(None)
    }

    if let Some(quiztype) = get_random_overdue_cooldown_item(conn, user.id)? {
        return Ok(Some(quiztype))
    }

    Ok(None)
}

fn get_q_or_e(conn: &PgConnection, user: &User, metrics: &UserMetrics) -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break
    || metrics.quizes_today >= metrics.max_quizes_today {
        return Ok(None)
    }


    if let Some(quiztype) = get_random_overdue_item(conn, user.id)? {
        return Ok(Some(quiztype))
    }

    if user::check_user_group(conn, user, "input_group")? {
        if let Some(q) = get_new_question(conn, user.id)? {
            return Ok(Some(QuizType::Question(q.id)))
        }
    }

    if user::check_user_group(conn, user, "output_group")? {
        if let Some(e) = get_new_exercise(conn, user.id)? {
            return Ok(Some(QuizType::Exercise(e.id)))
        }
    }

    Ok(None)
}

fn return_q_or_e(conn: &PgConnection, user: &User, quiztype: QuizType, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {

    metrics.quizes_today += 1;
    metrics.quizes_since_break += 1;

    match quiztype {
            QuizType::Question(id) => {
    
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
            QuizType::Exercise(id) => {
    
                let word = try_or!( load_word(conn, id)?,
                    else return Err(ErrorKind::DatabaseOdd("Bug: Not found despite being handed the ID? Concurrency error, possibly?").to_err()));
                let audio_file = audio::load_random_from_bundle(conn, word.audio_bundle)?;
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
            QuizType::Word(_) => unreachable!(),
        }
}

fn return_word(conn: &PgConnection, user: &User, the_word: Word, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {

    metrics.new_words_today += 1;
    metrics.new_words_since_break += 1;

    let audio_file = audio::load_random_from_bundle(&*conn, the_word.audio_bundle)?;
    let show_accents = user::check_user_group(conn, user, "show_accents")?;

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






/* MAIN LOGIC */

fn get_new_quiz_inner(conn : &PgConnection, user : &User, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {

    // Pending item first (items that were asked, but not answered because of loss of connection, user closing the session etc.)

    if let Some(quiztype) = get_pending_item(conn, user.id)? {
        return Ok(Some(quiztype));
    }

    
    // If the user is on break, just return a future due item and stop.

    if let Some(future) = check_break(conn, metrics)? {
        return Ok(Some(future));
    }


    // After that, question & exercise reviews that are overdue (except if they are on a cooldown period), and after that, new ones

    if let Some(quiztype) = get_q_or_e(conn, user, metrics)? {
        return return_q_or_e(conn, user, quiztype, metrics);
    }


    // No questions & exercises available at the moment. Introducing new words

    if let Some(the_word) = introduce_new_word(conn, metrics)? {
        return return_word(conn, user, the_word, metrics)
    }


    // If there's nothing else, time to show the cooled-down stuff!

    if let Some(quiztype) = get_cooldown_q_or_e(conn, user, metrics)? {
        return return_q_or_e(conn, user, quiztype, metrics)
    }

    Ok(None) // No words left to study
}







/* PUBLIC APIS */

pub fn get_new_quiz(conn : &PgConnection, user : &User) -> Result<Option<Quiz>> {
    use schema::user_metrics;
    
    let mut metrics : UserMetrics = user_metrics::table
        .filter(user_metrics::id.eq(user.id))
        .get_result(conn)?;

    let result = get_new_quiz_inner(conn, user, &mut metrics)?;

    let _ : UserMetrics = metrics.save_changes(conn)?;

    Ok(result)
}


pub fn get_next_quiz(conn : &PgConnection, user : &User, answer_enum: Answered)
    -> Result<Option<Quiz>>
{
    use schema::user_metrics;

    let mut metrics : UserMetrics = user_metrics::table
        .filter(user_metrics::id.eq(user.id))
        .get_result(conn)?;

    match answer_enum {
        Answered::W(answer_word) => {
            log_answer_word(conn, user, &answer_word)?;
        },
        Answered::E(exercise) => {
            log_answer_exercise(conn, user, &exercise, &metrics)?;
        },
        Answered::Q(answer) => {
            log_answer_question(conn, user, &answer, &metrics)?;
        },
    }

    let result = get_new_quiz_inner(conn, user, &mut metrics)?;

    let _ : UserMetrics = metrics.save_changes(conn)?;

    Ok(result)
}


