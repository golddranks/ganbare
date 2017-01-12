use super::*;
use rand::{Rng, thread_rng};
use diesel::expression::dsl::{all, any};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, RustcEncodable)]
pub enum Answered {
    W(WAnsweredData),
    Q(QAnsweredData),
    E(EAnsweredData),
}

#[derive(Debug, Clone, Copy)]
pub enum QuizType {
    Question(i32),
    Exercise(i32),
    Word(i32),
}

#[derive(Debug, Clone)]
pub enum Quiz {
    W(WordJson),
    E(ExerciseJson),
    Q(QuestionJson),
    F(FutureJson),
}

#[derive(RustcEncodable, Debug, Clone)]
pub struct FutureJson {
    pub quiz_type: &'static str,
    pub due_date: String,
}

#[derive(RustcEncodable, Debug, Clone)]
pub struct QuestionJson {
    pub quiz_type: &'static str,
    pub asked_id: i32,
    pub explanation: String,
    pub question: String,
    pub right_a: i32,
    pub answers: Vec<(i32, String)>,
}

#[derive(RustcEncodable, Debug, Clone)]
pub struct ExerciseJson {
    pub quiz_type: &'static str,
    pub event_name: Option<&'static str>,
    pub asked_id: i32,
    pub word: String,
    pub explanation: String,
    pub must_record: bool,
}

#[derive(RustcEncodable, Debug, Clone)]
pub struct WordJson {
    pub quiz_type: &'static str,
    pub asked_id: i32,
    pub word: String,
    pub explanation: String,
    pub show_accents: bool,
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
    use schema::{user_stats, pending_items, w_asked_data, w_answered_data, words};

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

    let mut stats: UserStats = user_stats::table
        .filter(user_stats::id.eq(user.id))
        .get_result(conn)?;

    stats.all_active_time_ms += answered.active_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.all_words += 1;
    let _: UserStats = stats.save_changes(conn)?;

    skill::log_by_id(conn, user.id, word.skill_nugget, 1)?;

    Ok(())
}

fn log_answer_question(conn : &PgConnection, user : &User, answered: &QAnsweredData, metrics: &UserMetrics) -> Result<(QAskedData, QuestionData, DueItem)> {
    use schema::{user_stats, pending_items, q_asked_data, q_answered_data, due_items, question_data, quiz_questions};

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

    let mut stats: UserStats = user_stats::table
        .filter(user_stats::id.eq(user.id))
        .get_result(conn)?;

    stats.all_active_time_ms += answered.full_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.quiz_all_times += 1;
    if correct {
        stats.quiz_correct_times += 1;
    }
    let _: UserStats = stats.save_changes(conn)?;


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
    use schema::{user_stats, pending_items, e_asked_data, e_answered_data, due_items, exercise_data, exercises};

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

    let mut stats: UserStats = user_stats::table
        .filter(user_stats::id.eq(user.id))
        .get_result(conn)?;

    stats.all_active_time_ms += answered.active_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.quiz_all_times += 1;
    if answered.answer_level > 0 {
        stats.quiz_correct_times += 1;
    }
    let _: UserStats = stats.save_changes(conn)?;


    // If the answer was wrong, register a new pending question with the same specs right away for a follow-up review
    if answered.answer_level == 0 {

        let pending_item = new_pending_item(conn, user.id, QuizType::Exercise(pending_item.audio_file_id))?;
        let asked_data = EAskedData {
            id: pending_item.id,
            exercise_id: asked.exercise_id,
            word_id: asked.word_id,
        };
        register_future_e_answer(conn, &asked_data)?;

    }

    let exercise : Exercise = exercises::table
                                .filter(exercises::id.eq(asked.exercise_id))
                                .get_result(conn)?;

    let exercisedata : Option<(ExerciseData, DueItem)> = exercise_data::table
                                        .inner_join(due_items::table)
                                        .filter(due_items::user_id.eq(user.id))
                                        .filter(exercise_data::exercise_id.eq(asked.exercise_id))
                                        .get_result(&*conn)
                                        .optional()?;

    // Update the data for this word exercise (due date, statistics etc.)
    Ok(if let Some((exercisedata, due_item)) = exercisedata {

        let due_item = log_answer_due_item(conn, due_item, exercise.skill_id, correct, metrics)?;

        (exercisedata, due_item)

    } else { // New!

        let due_item = log_answer_new_due_item(conn, user.id, "exercise", exercise.skill_id, correct, metrics)?;

        let exercisedata = ExerciseData {
            due: due_item.id,
            exercise_id: asked.exercise_id,
        };
        let exercisedata = diesel::insert(&exercisedata)
            .into(exercise_data::table)
            .get_result(conn)
            .chain_err(|| "Couldn't save the question tally data to database!")?;
        (exercisedata, due_item)
    })
}










/* FETCHING & CHOOSING QUESTIONS */


pub fn load_question(conn : &PgConnection, id: i32 ) -> Result<Option<(QuizQuestion, Vec<Answer>, Vec<AudioBundle>)>> {
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
    
    Ok(Some((qq, aas, q_bundles)))
}

pub fn load_exercise(conn : &PgConnection, id: i32 ) -> Result<Option<(Exercise, Vec<ExerciseVariant>, Vec<Word>)>> {
    use schema::{exercises, exercise_variants, words};

    let qq : Option<Exercise> = exercises::table
        .filter(exercises::id.eq(id))
        .get_result(&*conn)
        .optional()?;

    let qq = try_or!{ qq, else return Ok(None) };

    let (aas, words) : (Vec<ExerciseVariant>, Vec<Word>) = exercise_variants::table
        .inner_join(words::table)
        .filter(exercise_variants::exercise_id.eq(qq.id))
        .load(&*conn)?
        .into_iter().unzip();

    Ok(Some((qq, aas, words)))
}

fn load_word(conn : &PgConnection, id: i32 ) -> Result<Option<Word>> {
    use schema::{words};

    Ok(words::table
        .filter(words::id.eq(id))
        .get_result(&*conn)
        .optional()?)
}

fn choose_next_due_item(conn : &PgConnection, user_id: i32) -> Result<Option<(DueItem, QuizType)>> {
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
                ((di, Some(qdata)), (_, None)) => (di, QuizType::Question(qdata.question_id)),
                ((_, None), (di, Some(edata))) => (di, QuizType::Exercise(edata.exercise_id)),
                e => { println!("WHY? {:?}", e); unreachable!()},
            });

    Ok(due_item)
}

pub fn count_overdue_items(conn : &PgConnection, user_id: i32) -> Result<i64> {
    use schema::{due_items};

    let count: i64 = due_items::table
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .count()
        .get_result(conn)?;

    Ok(count)
}

fn choose_random_overdue_item(conn : &PgConnection, user_id: i32) -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due: Option<DueItem> = due_items::table
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .filter(due_items::cooldown_delay.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(match due {
        Some(ref due) if due.item_type == "question" => {
            Some(QuizType::Question(
                question_data::table
                    .filter(question_data::due.eq(due.id))
                    .get_result::<QuestionData>(conn)?.question_id
            ))
        },
        Some(ref due) if due.item_type == "exercise" => {
            Some(QuizType::Exercise(
                exercise_data::table
                    .filter(exercise_data::due.eq(due.id))
                    .get_result::<ExerciseData>(conn)?.exercise_id
            ))
        },
        Some(_) => return Err(ErrorKind::DatabaseOdd("Database contains due_item with an odd item_type value!").into()),
        None => None,
    })
}

fn choose_random_overdue_item_include_cooldown(conn : &PgConnection, user_id: i32) -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due: Option<DueItem> = due_items::table
        .filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::UTC::now()))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(match due {
        Some(ref due) if due.item_type == "question" => {
            Some(QuizType::Question(
                question_data::table
                    .filter(question_data::due.eq(due.id))
                    .get_result::<QuestionData>(conn)?.question_id
            ))
        },
        Some(ref due) if due.item_type == "exercise" => {
            Some(QuizType::Exercise(
                exercise_data::table
                    .filter(exercise_data::due.eq(due.id))
                    .get_result::<ExerciseData>(conn)?.exercise_id
            ))
        },
        Some(_) => return Err(ErrorKind::DatabaseOdd("Database contains due_item with an odd item_type value!").into()),
        None => None,
    })
}

fn choose_new_question(conn : &PgConnection, user_id : i32) -> Result<Option<QuizQuestion>> {
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

fn choose_new_exercise(conn : &PgConnection, user_id : i32) -> Result<Option<Exercise>> {
    use schema::{exercises, exercise_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(exercise_data::table)
        .select(exercise_data::exercise_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
        .filter(skill_data::skill_level.gt(1)) // Take only skills with level >= 2 (=both words introduced) before
        .filter(skill_data::user_id.eq(user_id));

    let new_exercise : Option<Exercise> = exercises::table
        .filter(exercises::id.ne(all(dues)))
        .filter(exercises::skill_id.eq(any(skills)))
        .filter(exercises::published.eq(true))
        .order(sql::random)
        .first(conn)
        .optional()?;

    Ok(new_exercise)
}

fn choose_cooldown_q_or_e(conn: &PgConnection, user: &User, metrics: &UserMetrics) -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break
    || metrics.quizes_today >= metrics.max_quizes_today
    || metrics.break_until > chrono::UTC::now() {
        return Ok(None)
    }

    if let Some(quiztype) = choose_random_overdue_item_include_cooldown(conn, user.id)? {
        return Ok(Some(quiztype))
    }

    Ok(None)
}

fn choose_new_q_or_e(conn: &PgConnection, user_id: i32) -> Result<Option<QuizType>> {

    if user::check_user_group(conn, user_id, "input_group")? {
        if let Some(q) = choose_new_question(conn, user_id)? {
            return Ok(Some(QuizType::Question(q.id)))
        }
    }

    if user::check_user_group(conn, user_id, "output_group")? {
        if let Some(e) = choose_new_exercise(conn, user_id)? {
            return Ok(Some(QuizType::Exercise(e.id)))
        }
    }
    Ok(None)
}

fn choose_q_or_e(conn: &PgConnection, user: &User, metrics: &UserMetrics) -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break
    || metrics.quizes_today >= metrics.max_quizes_today
    || metrics.break_until > chrono::UTC::now() {
        debug!("Enough quizes for today. The break/daily limits are full.");
        return Ok(None)
    }


    if let Some(quiztype) = choose_random_overdue_item(conn, user.id)? {
        debug!("There is an overdue quiz item; presenting that.");
        return Ok(Some(quiztype))
    }

    if let Some(quiztype) = choose_new_q_or_e(conn, user.id)? {
        debug!("No overdue quiz items; presenting a new quiz.");
        return Ok(Some(quiztype))
    }
    
    debug!("No quiz items at all; returning.");

    Ok(None)
}

fn choose_new_random_word(conn : &PgConnection, user_id : i32) -> Result<Option<Word>> {
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

fn choose_new_paired_word(conn : &PgConnection, user_id : i32) -> Result<Option<Word>> {
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

fn choose_new_word(conn : &PgConnection, metrics: &mut UserMetrics) -> Result<Option<Word>> {


    if metrics.new_words_since_break >= metrics.max_words_since_break
    || metrics.new_words_today >= metrics.max_words_today
    || metrics.break_until > chrono::UTC::now() {
        debug!("Enough words for today. The break/daily limits are full.");
        return Ok(None)
    }

    let user_id = metrics.id;

    if let Some(paired_word) = choose_new_paired_word(conn, user_id)? {
        debug!("There is a yet-unintroduced pair word; presenting that.");
        return Ok(Some(paired_word));
    }
    
    if let Some(rand_word) = choose_new_random_word(conn, user_id)? {
        debug!("No paired words, returning a randomly chosen word.");
        return Ok(Some(rand_word));
    }

    debug!("No words at all; returning.");
    Ok(None)
}

fn clear_limits(conn: &PgConnection, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {
    use schema::user_stats;

    if chrono::UTC::now() < metrics.break_until {
        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson{ quiz_type: "future", due_date: due_string })));
    }

    // This is important because we have to zero the counts once every day even though we wouldn't break a single time!
    // We are having 1 AM as the change point of the day
    // (That's 3 AM in Finland, where the users of this app are likely to reside)
    if chrono::UTC::now() > (metrics.today.date().and_hms(1, 0, 0) + chrono::Duration::hours(24)) {

        let mut stats: UserStats = user_stats::table
            .filter(user_stats::id.eq(metrics.id))
            .get_result(conn)?;
    
        stats.days_used += 1;
        let _: UserStats = stats.save_changes(conn)?;
    
        metrics.today = chrono::UTC::today().and_hms(1, 0, 0);
        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
    }
    Ok(None)
}

pub fn things_left_to_do(conn: &PgConnection, user_id: i32) -> Result<(Option<chrono::DateTime<chrono::UTC>>, bool, bool)> {

    let next_existing_due = choose_next_due_item(conn, user_id)?.map(|(due_item, _)| due_item.due_date);
    let no_new_words = choose_new_random_word(conn, user_id)?.is_none();
    let no_new_quizes =  choose_new_q_or_e(conn, user_id)?.is_none();

    Ok((next_existing_due, no_new_words, no_new_quizes))
}


fn check_break(conn: &PgConnection, user: &User, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {
    use std::cmp::{max};
    use chrono::Duration;

    let user_id = user.id;

    let (next_existing_due, no_new_words, no_new_quizes) = things_left_to_do(conn, user_id)?;

    debug!("No new quizes available: {:?} (either limited by lack of new words or by lack of quizes themselves)", no_new_quizes);
    debug!("No due quizes available: {:?}", next_existing_due.is_none());

    if no_new_words && no_new_quizes && next_existing_due.is_none() {
        return Ok(None) // No words to study
    }

    let current_overdue = choose_random_overdue_item(conn, user_id)?;

    let words = metrics.new_words_today >= metrics.max_words_today || no_new_words;
    let new_quizes = metrics.quizes_today >= metrics.max_quizes_today || no_new_quizes;
    let due_quizes = metrics.quizes_today >= metrics.max_quizes_today || current_overdue.is_none();

    if words && new_quizes && due_quizes
    {
        debug!("Starting a break because the daily limits are full.");

        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
        metrics.break_until = metrics.today + Duration::hours(24);

        if no_new_words && no_new_quizes { // Nothing else left but due items – so no use breaking until there is some available
            metrics.break_until = max(metrics.break_until, next_existing_due.unwrap_or(chrono::date::MAX.and_hms(0,0,0,)));
        }

        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson{ quiz_type: "future", due_date: due_string })));
    }

    let words = metrics.new_words_since_break >= metrics.max_words_since_break || no_new_words;
    let new_quizes = metrics.quizes_since_break >= metrics.max_quizes_since_break || no_new_quizes;
    let due_quizes = metrics.quizes_since_break >= metrics.max_quizes_since_break || current_overdue.is_none();

    // Start a break if the limits are full.

    if words && new_quizes && due_quizes
    {
        debug!("Starting a break because the break limits are full.");

        let time_since_last_break = chrono::UTC::now() - metrics.break_until;
    
        let discounted_breaktime = max(Duration::seconds(0),
                Duration::seconds(metrics.break_length as i64) - time_since_last_break);

        metrics.break_until = chrono::UTC::now()
                                + discounted_breaktime;

        if no_new_words && no_new_quizes { // Nothing else left but due items – so no use breaking until there is some available
            metrics.break_until = max(metrics.break_until, next_existing_due.unwrap_or(chrono::date::MAX.and_hms(0,0,0,)));
        }

        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;

        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson{ quiz_type: "future", due_date: due_string })));
    }

    unreachable!("check_break. Either the break limits or daily limits should be full and those code paths return, so this should never happen.");
}


fn ask_new_question(conn: &PgConnection, id: i32) -> Result<(QuizQuestion, i32, Vec<Answer>, i32)> {
    use rand::Rng;

    let (question, answers, q_audio_bundles) = try_or!{ load_question(conn, id)?,
                else return Err(ErrorKind::DatabaseOdd("This function was called on the premise that the data exists!").to_err()) };
    
    let mut rng = thread_rng();
    let random_answer_index = rng.gen_range(0, answers.len());
    let right_answer_id = answers[random_answer_index].id;
    let q_audio_bundle = &q_audio_bundles[random_answer_index];

    let q_audio_file = audio::load_random_from_bundle(conn, q_audio_bundle.id)?;

    Ok((question, right_answer_id, answers, q_audio_file.id))
}

fn ask_new_exercise(conn: &PgConnection, id: i32) -> Result<(Exercise, Word, i32)> {
    let (exercise, _, mut words) = try_or!( load_exercise(conn, id)?,
                else return Err(ErrorKind::DatabaseOdd("This function was called on the premise that the data exists!").to_err()) );

    let random_answer_index = thread_rng().gen_range(0, words.len());
    let word = words.swap_remove(random_answer_index);

    let audio_file = audio::load_random_from_bundle(conn, word.audio_bundle)?;

    Ok((exercise, word, audio_file.id))
}

pub fn pi_to_quiz(conn: &PgConnection, pi: &PendingItem) -> Result<Quiz> {
    use schema::{q_asked_data, e_asked_data, w_asked_data};

    Ok(match pi {
        ref pi if pi.item_type == "question" => {

            let asked: QAskedData = q_asked_data::table
                .filter(q_asked_data::id.eq(pi.id))
                .get_result(conn)?;

            let (question, answers, _) = try_or!{ load_question(conn, asked.question_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            let mut answer_choices: Vec<_> = answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
            thread_rng().shuffle(&mut answer_choices);

            Quiz::Q(QuestionJson {
                quiz_type: "question",
                asked_id: pi.id,
                explanation: question.q_explanation,
                question: question.question_text,
                right_a: asked.correct_qa_id,
                answers: answer_choices,
            })

        },
        ref pi if pi.item_type == "exercise" => {

            let asked: EAskedData = e_asked_data::table
                .filter(e_asked_data::id.eq(pi.id))
                .get_result(conn)?;

            let word = try_or!{ load_word(conn, asked.word_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::E(ExerciseJson {
                quiz_type: "exercise",
                event_name: None,
                asked_id: pi.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
                must_record: false,
            })

        },
        ref pi if pi.item_type == "word" => {

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
        _ => unreachable!("Bug: There is only three kinds of quiz types!"),
    })
}

pub fn return_pending_item(conn: &PgConnection, user_id: i32) -> Result<Option<Quiz>> {
    use schema::{pending_items};

    let pending_item: Option<PendingItem> = pending_items::table
        .filter(pending_items::user_id.eq(user_id))
        .filter(pending_items::pending.eq(true))
        .get_result(conn)
        .optional()?;

    let quiz_type = match pending_item {
        Some(ref pi) => pi_to_quiz(conn, pi)?,
        None => return Ok(None),
    };

    debug!("There was a pending item! Returning it.");

    Ok(Some(quiz_type))
}

pub fn return_q_or_e(conn: &PgConnection, user: &User, quiztype: QuizType) -> Result<Option<Quiz>> {

    match quiztype {
            QuizType::Question(id) => {
    
                let (question, right_a_id, answers, q_audio_id) = ask_new_question(conn, id)?;
    
                let pending_item = new_pending_item(conn, user.id, QuizType::Question(q_audio_id))?;
    
                let asked_data = QAskedData {
                    id: pending_item.id,
                    question_id: question.id,
                    correct_qa_id: right_a_id,
                };

                register_future_q_answer(conn, &asked_data)?;
    
                let mut answer_choices: Vec<_> = answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
                thread_rng().shuffle(&mut answer_choices);
    
                let quiz_json = QuestionJson {
                    quiz_type: "question",
                    asked_id: pending_item.id,
                    explanation: question.q_explanation,
                    question: question.question_text,
                    right_a: right_a_id,
                    answers: answer_choices,
                };
                
                return Ok(Some(Quiz::Q(quiz_json)))
        
            },
            QuizType::Exercise(id) => {
    
                let (exercise, word, audio_id) = ask_new_exercise(conn, id)?;

                let pending_item = new_pending_item(conn, user.id, QuizType::Exercise(audio_id))?;
    
                let asked_data = EAskedData {
                    id: pending_item.id,
                    exercise_id: exercise.id,
                    word_id: word.id,
                };
    
                register_future_e_answer(conn, &asked_data)?;
    
                let quiz_json = ExerciseJson{
                    quiz_type: "exercise",
                    event_name: None,
                    asked_id: pending_item.id,
                    word: word.word.nfc().collect::<String>(),
                    explanation: word.explanation,
                    must_record: false,
                };
    
                return Ok(Some(Quiz::E(quiz_json)))
            },
            QuizType::Word(_) => unreachable!(),
        }
}

pub fn return_word(conn: &PgConnection, user: &User, the_word: Word) -> Result<Option<Quiz>> {

    let audio_file = audio::load_random_from_bundle(&*conn, the_word.audio_bundle)?;
    let show_accents = user::check_user_group(conn, user.id, "show_accents")?;

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

pub fn get_word_id(conn: &PgConnection, word: &str) -> Result<i32> {
    use schema::words;

    Ok(words::table
        .filter(words::word.eq(word))
        .select(words::id)
        .get_result(conn)?)
}

pub fn get_exercise_id(conn: &PgConnection, skill_summary: &str) -> Result<i32> {
    use schema::{exercises, skill_nuggets};

    Ok(exercises::table
        .inner_join(skill_nuggets::table)
        .filter(skill_nuggets::skill_summary.eq(skill_summary).and(exercises::skill_level.lt(3)))
        .select(exercises::id)
        .get_result(conn)?)
}

pub fn get_question_id(conn: &PgConnection, skill_summary: &str) -> Result<i32> {
    use schema::{quiz_questions, skill_nuggets};

    Ok(quiz_questions::table
        .inner_join(skill_nuggets::table)
        .filter(skill_nuggets::skill_summary.eq(skill_summary).and(quiz_questions::skill_level.lt(3)))
        .select(quiz_questions::id)
        .get_result(conn)?)
}

pub enum QuizSerialized {
    Word(&'static str, i32),
    Question(&'static str, i32, i32),
    Exercise(&'static str, i32),
}

pub fn test_item(conn: &PgConnection, user: &User, quiz_str: &QuizSerialized) -> Result<(Quiz, i32)> {
    let pending_item;
    let test_item = match quiz_str {
        &QuizSerialized::Word(ref s, audio_id) => {
            pending_item = new_pending_item(conn, user.id, QuizType::Word(audio_id))?;
        
            let asked_data = WAskedData {
                id: pending_item.id,
                word_id: quiz::get_word_id(conn, s).chain_err(|| format!("Word {} not found", s))?,
                show_accents: false,
            };
        
            register_future_w_answer(conn, &asked_data)?;

            let word = try_or!{ load_word(conn, asked_data.word_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::W(WordJson {
                quiz_type: "word",
                asked_id: pending_item.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
                show_accents: asked_data.show_accents,
            })
        },
        &QuizSerialized::Question(ref s, audio_id, variant_id) => {

            pending_item = new_pending_item(conn, user.id, QuizType::Question(audio_id))?;

            let asked_data = QAskedData {
                id: pending_item.id,
                question_id: quiz::get_question_id(conn, s).chain_err(|| format!("Question {} not found", s))?,
                correct_qa_id: variant_id,
            };
            register_future_q_answer(conn, &asked_data)?;

            let (question, answers, _) = try_or!{ load_question(conn, asked_data.question_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            let mut answer_choices: Vec<_> = answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
            thread_rng().shuffle(&mut answer_choices);


            Quiz::Q(QuestionJson {
                quiz_type: "question",
                asked_id: pending_item.id,
                explanation: question.q_explanation,
                question: question.question_text,
                right_a: asked_data.correct_qa_id,
                answers: answer_choices,
            })

        },
        &QuizSerialized::Exercise(ref word, audio_id) => {

            pending_item = new_pending_item(conn, user.id, QuizType::Exercise(audio_id))?;
            let exercise_name = word.replace('*', "").replace('・', "");

            let asked_data = EAskedData {
                id: pending_item.id,
                exercise_id: quiz::get_exercise_id(conn, &exercise_name).chain_err(|| format!("Exercise {} not found", exercise_name))?,
                word_id: quiz::get_word_id(conn, word).chain_err(|| format!("Word {} not found", word))?,
            };

            register_future_e_answer(conn, &asked_data)?;

            let word = try_or!{ load_word(conn, asked_data.word_id)?,
                else return Err(ErrorKind::DatabaseOdd("Bug: If the item was set pending in the first place, it should exists!").to_err()) };

            Quiz::E(ExerciseJson {
                quiz_type: "exercise",
                event_name: None,
                asked_id: pending_item.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
                must_record: false,
            })
        },
    };

    debug!("There was a test item! Returning it.");

    Ok((test_item, pending_item.id))
}



/* MAIN LOGIC */

fn get_new_quiz_inner(conn : &PgConnection, user : &User, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {


    // Pending item first (items that were asked, but not answered because of loss of connection, user closing the session etc.)

    if let Some(pending_quiz) = return_pending_item(conn, user.id)? {
        return Ok(Some(pending_quiz));
    }

     // Clear the per-day limits if it's tomorrow already and stop if we are in the middle of a break
    if let Some(future) = clear_limits(conn, metrics)? {
        return Ok(Some(future));
    }
    
    // After that, question & exercise reviews that are overdue (except if they are on a cooldown period), and after that, new ones

    if let Some(quiztype) = choose_q_or_e(conn, user, metrics)? {
        metrics.quizes_today += 1;
        metrics.quizes_since_break += 1;
        return return_q_or_e(conn, user, quiztype);
    }


    // No questions & exercises available at the moment. Introducing new words

    if let Some(the_word) = choose_new_word(conn, metrics)? {
        metrics.new_words_today += 1;
        metrics.new_words_since_break += 1;
        return return_word(conn, user, the_word)
    }


    // If there's nothing else, time to show the cooled-down stuff!

    if let Some(quiztype) = choose_cooldown_q_or_e(conn, user, metrics)? {
        metrics.quizes_today += 1;
        metrics.quizes_since_break += 1;
        return return_q_or_e(conn, user, quiztype)
    }

    // There seems to be nothig to do?! Either there is no more words to study or the limits are full.

    if let Some(future) = check_break(conn, user, metrics)? {
        return Ok(Some(future));
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


