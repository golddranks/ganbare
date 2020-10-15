use super::*;
use rand::{Rng, thread_rng, seq::SliceRandom};
use unicode_normalization::UnicodeNormalization;
use serde::Serialize;
use error_chain::bail;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Serialize, Debug, Clone)]
pub struct FutureJson {
    pub quiz_type: &'static str,
    pub due_date: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct QuestionJson {
    pub quiz_type: &'static str,
    pub asked_id: i32,
    pub explanation: String,
    pub question: String,
    pub right_a: i32,
    pub answers: Vec<(i32, String)>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ExerciseJson {
    pub quiz_type: &'static str,
    pub event_name: &'static str,
    pub asked_id: i32,
    pub word: String,
    pub explanation: String,
    pub must_record: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct WordJson {
    pub quiz_type: &'static str,
    pub asked_id: i32,
    pub word: String,
    pub explanation: String,
    pub show_accents: bool,
}












/* ANSWERING */


fn new_pending_item(conn: &Connection,
                    user_id: i32,
                    quiz_n_audio: QuizType,
                    test_item: bool)
                    -> Result<PendingItem> {
    use schema::pending_items;
    use self::QuizType::*;

    let (item_type, audio_file_id) = match quiz_n_audio {
        Question(id) => ("question", id),
        Exercise(id) => ("exercise", id),
        Word(id) => ("word", id),
    };

    Ok(diesel::insert_into(pending_items::table).values(&NewPendingItem {
                           user_id: user_id,
                           audio_file_id: audio_file_id,
                           item_type: item_type,
                           test_item: test_item,
                       })
               .get_result(&**conn)?)
}

fn register_future_q_answer(conn: &Connection, data: &QAskedData) -> Result<()> {
    use schema::q_asked_data;

    diesel::insert_into(q_asked_data::table).values(data).execute(&**conn)?;
    Ok(())
}

fn register_future_e_answer(conn: &Connection, data: &EAskedData) -> Result<()> {
    use schema::e_asked_data;

    diesel::insert_into(e_asked_data::table).values(data).execute(&**conn)?;
    Ok(())
}

fn register_future_w_answer(conn: &Connection, data: &WAskedData) -> Result<()> {
    use schema::w_asked_data;

    diesel::insert_into(w_asked_data::table).values(data).execute(&**conn)?;
    Ok(())
}

fn log_answer_due_item(conn: &Connection,
                       mut due_item: DueItem,
                       skill_id: i32,
                       correct: bool,
                       metrics: &UserMetrics)
                       -> Result<DueItem> {
    use std::cmp::max;

    due_item.correct_streak_this_time = if correct {
        due_item.correct_streak_this_time + 1
    } else {
        0
    };
    due_item.cooldown_delay = chrono::offset::Utc::now() +
                              chrono::Duration::seconds(metrics.cooldown_delay as i64);

    if due_item.correct_streak_this_time >= metrics.streak_limit {
        due_item.correct_streak_this_time = 0;
        due_item.correct_streak_overall = if correct {
            due_item.correct_streak_overall + 1
        } else {
            0
        };
        due_item.due_delay = if correct {
            max(due_item.due_delay * metrics.delay_multiplier,
                metrics.initial_delay)
        } else {
            0
        };
        due_item.due_date = chrono::offset::Utc::now() +
                            chrono::Duration::seconds(due_item.due_delay as i64);
        if due_item.correct_streak_overall >= metrics.streak_skill_bump_criteria {

            debug!("Skill bump because of correct_streak_overall > the criteria! Skill: {} Of \
                    user: {} Bumped by: {}",
                   skill_id,
                   due_item.user_id,
                   1);

            skill::log_by_id(conn, due_item.user_id, skill_id, 1)?;
        };
    }

    Ok(due_item.save_changes(&**conn)?)
}

fn log_answer_new_due_item(conn: &Connection,
                           user_id: i32,
                           item_type: &str,
                           skill_id: i32,
                           correct: bool,
                           metrics: &UserMetrics)
                           -> Result<DueItem> {
    use schema::due_items;

    // Diesel doesn't have UPSERT so we have to initialize separately.

    debug!("Skill bump because of first time bonus! Skill: {} Of user: {} Bumped by: {}",
           skill_id,
           user_id,
           1);

    skill::log_by_id(conn, user_id, skill_id, 1)?; // First time bonus!

    let new_due_item = NewDueItem {
        user_id: user_id,
        correct_streak_this_time: 0,
        correct_streak_overall: 0,
        due_date: chrono::offset::Utc::now(),
        due_delay: 0,
        cooldown_delay: chrono::offset::Utc::now(),
        item_type: item_type,
    };

    let due_item: DueItem = diesel::insert_into(due_items::table).values(&new_due_item)
        .get_result(&**conn)?;

    Ok(log_answer_due_item(conn, due_item, skill_id, correct, metrics)?)
}

fn log_answer_word(conn: &Connection, user_id: i32, answered: &WAnsweredData) -> Result<()> {
    use schema::{user_stats, pending_items, w_asked_data, w_answered_data, words};

    let (mut pending_item, asked): (PendingItem, WAskedData) =
        pending_items::table.inner_join(w_asked_data::table)
            .filter(pending_items::id.eq(answered.id))
            .get_result(&**conn)?;

    if !pending_item.pending {
        info!("User is trying to ack twice the same word! Ignoring the later answer.");
        return Ok(());
    }

    // This Q&A is now considered done
    pending_item.pending = false;
    let _: PendingItem = pending_item.save_changes(&**conn)?;

    diesel::insert_into(w_answered_data::table).values(answered).execute(&**conn)?;

    let word: Word = words::table.filter(words::id.eq(asked.word_id)).get_result(&**conn)?;

    let mut stats: UserStats = user_stats::table.filter(user_stats::id.eq(user_id))
        .get_result(&**conn)?;

    stats.all_active_time_ms += answered.active_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.all_words += 1;
    let _: UserStats = stats.save_changes(&**conn)?;

    debug!("Skill bump because of newly learned word! Skill: {} Of user: {} Bumped by: {}",
           word.skill_nugget,
           user_id,
           1);

    skill::log_by_id(conn, user_id, word.skill_nugget, 1)?;

    Ok(())
}

fn log_answer_question(conn: &Connection,
                       user_id: i32,
                       answered: &QAnsweredData,
                       metrics: &UserMetrics)
                       -> Result<()> {
    use schema::{user_stats, pending_items, q_asked_data, q_answered_data, due_items, question_data,
                 quiz_questions};

    let (mut pending_item, asked): (PendingItem, QAskedData) =
        pending_items::table.inner_join(q_asked_data::table)
            .filter(pending_items::id.eq(answered.id))
            .get_result(&**conn)?;

    if !pending_item.pending {
        info!("User is trying to answer twice to the same question! Ignoring the later answer.");
        return Ok(());
    }

    // This Q&A is now considered done
    pending_item.pending = false;
    let _: PendingItem = pending_item.save_changes(&**conn)?;

    let correct = asked.correct_qa_id == answered.answered_qa_id.unwrap_or(-1);

    diesel::insert_into(q_answered_data::table).values(answered).execute(&**conn)?;

    let mut stats: UserStats = user_stats::table.filter(user_stats::id.eq(user_id))
        .get_result(&**conn)?;

    stats.all_active_time_ms += answered.full_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.quiz_all_times += 1;
    if correct {
        stats.quiz_correct_times += 1;
    }
    let _: UserStats = stats.save_changes(&**conn)?;


    // If the answer was wrong, register a new pending question
    // with the same specs right away for a follow-up review
    if !correct {

        let pending_item = new_pending_item(conn,
                                            user_id,
                                            QuizType::Question(pending_item.audio_file_id),
                                            false)?;
        let asked_data = QAskedData {
            id: pending_item.id,
            question_id: asked.question_id,
            correct_qa_id: asked.correct_qa_id,
        };
        register_future_q_answer(conn, &asked_data)?;

    }


    // Update the data for the question (Diesel doesn't support UPSERT so we have to branch)

    let question: QuizQuestion =
        quiz_questions::table.filter(quiz_questions::id.eq(asked.question_id)).get_result(&**conn)?;

    let questiondata: Option<(QuestionData, DueItem)> =
        question_data::table.inner_join(due_items::table)
            .filter(due_items::user_id.eq(user_id))
            .filter(question_data::question_id.eq(asked.question_id))
            .get_result(&**conn)
            .optional()?;

    // Update the data for this question (due date, statistics etc.)
    Ok(if let Some((_, due_item)) = questiondata {

           log_answer_due_item(conn, due_item, question.skill_id, correct, metrics)?;

       } else {
           // New!

           let due_item = log_answer_new_due_item(conn,
                                               user_id,
                                               "question",
                                               question.skill_id,
                                               correct,
                                               metrics)?;

           let questiondata = QuestionData {
            question_id: asked.question_id,
            due: due_item.id,
        };
           let _: QuestionData = diesel::insert_into(question_data::table).values(&questiondata)
            .get_result(&**conn)?;
       })
}

fn log_answer_exercise(conn: &Connection,
                       user_id: i32,
                       answered: &EAnsweredData,
                       metrics: &UserMetrics)
                       -> Result<()> {
    use schema::{user_stats, pending_items, e_asked_data, e_answered_data, due_items, exercise_data,
                 exercises};

    let correct = answered.answer_level > 0;

    let (mut pending_item, asked): (PendingItem, EAskedData) =
        pending_items::table.inner_join(e_asked_data::table)
            .filter(pending_items::id.eq(answered.id))
            .get_result(&**conn)?;

    if !pending_item.pending {
        info!("User is trying to answer twice to the same excercise! Ignoring the later answer.");
        return Ok(());
    }

    // This Q&A is now considered done
    pending_item.pending = false;
    let _: PendingItem = pending_item.save_changes(&**conn)?;

    diesel::insert_into(e_answered_data::table).values(answered).execute(&**conn)?;

    let mut stats: UserStats = user_stats::table.filter(user_stats::id.eq(user_id))
        .get_result(&**conn)?;

    stats.all_active_time_ms += answered.active_answer_time_ms as i64;
    stats.all_spent_time_ms += answered.full_spent_time_ms as i64;
    stats.quiz_all_times += 1;
    if answered.answer_level > 0 {
        stats.quiz_correct_times += 1;
    }
    let _: UserStats = stats.save_changes(&**conn)?;


    // If the answer was wrong, register a new pending question
    // with the same specs right away for a follow-up review
    if answered.answer_level == 0 {

        let pending_item = new_pending_item(conn,
                                            user_id,
                                            QuizType::Exercise(pending_item.audio_file_id),
                                            false)?;
        let asked_data = EAskedData {
            id: pending_item.id,
            exercise_id: asked.exercise_id,
            word_id: asked.word_id,
        };
        register_future_e_answer(conn, &asked_data)?;

    }

    let exercise: Exercise = exercises::table.filter(exercises::id.eq(asked.exercise_id))
        .get_result(&**conn)?;

    let exercisedata: Option<(ExerciseData, DueItem)> =
        exercise_data::table.inner_join(due_items::table)
            .filter(due_items::user_id.eq(user_id))
            .filter(exercise_data::exercise_id.eq(asked.exercise_id))
            .get_result(&**conn)
            .optional()?;

    // Update the data for this word exercise (due date, statistics etc.)
    Ok(if let Some((_, due_item)) = exercisedata {

           log_answer_due_item(conn, due_item, exercise.skill_id, correct, metrics)?;

       } else {
           // New!

           let due_item = log_answer_new_due_item(conn,
                                               user_id,
                                               "exercise",
                                               exercise.skill_id,
                                               correct,
                                               metrics)?;

           let exercisedata = ExerciseData {
            due: due_item.id,
            exercise_id: asked.exercise_id,
        };
           let _: ExerciseData =
            diesel::insert_into(exercise_data::table).values(&exercisedata)
                .get_result(&**conn)
                .chain_err(|| "Couldn't save the question tally data to database!")?;
       })
}










/* FETCHING & CHOOSING QUESTIONS */


pub fn load_question(conn: &Connection,
                     id: i32)
                     -> Result<Option<(QuizQuestion, Vec<Answer>, Vec<AudioBundle>)>> {
    use schema::{quiz_questions, question_answers, audio_bundles};

    let qq: Option<QuizQuestion> = quiz_questions::table.filter(quiz_questions::id.eq(id))
        .get_result(&**conn)
        .optional()?;

    let qq = try_or!{ qq, else return Ok(None) };

    let (aas, q_bundles): (Vec<Answer>, Vec<AudioBundle>) =
        question_answers::table.inner_join(audio_bundles::table
                .on(question_answers::q_audio_bundle.eq(audio_bundles::id)))
            .filter(question_answers::question_id.eq(qq.id))
            .load(&**conn)?
            .into_iter()
            .unzip();

    Ok(Some((qq, aas, q_bundles)))
}

pub fn load_exercise(conn: &Connection,
                     id: i32)
                     -> Result<Option<(Exercise, Vec<ExerciseVariant>, Vec<Word>)>> {
    use schema::{exercises, exercise_variants, words};

    let qq: Option<Exercise> = exercises::table.filter(exercises::id.eq(id))
        .get_result(&**conn)
        .optional()?;

    let qq = try_or!{ qq, else return Ok(None) };

    let (aas, words): (Vec<ExerciseVariant>, Vec<Word>) =
        exercise_variants::table.inner_join(words::table)
            .filter(exercise_variants::exercise_id.eq(qq.id))
            .load(&**conn)?
            .into_iter()
            .unzip();

    Ok(Some((qq, aas, words)))
}

fn load_word(conn: &Connection, id: i32) -> Result<Option<Word>> {
    use schema::words;

    Ok(words::table.filter(words::id.eq(id))
           .get_result(&**conn)
           .optional()?)
}

fn choose_next_due_item(conn: &Connection, user_id: i32) -> Result<Option<(DueItem, QuizType)>> {
    use schema::{due_items, question_data, exercise_data};

    let due_questions: Option<(DueItem, Option<QuestionData>)> =
        due_items::table.left_outer_join(question_data::table)
            .filter(due_items::user_id.eq(user_id))
            .order(due_items::due_date.asc())
            .first(&**conn)
            .optional()?;

    let due_exercises: Option<(DueItem, Option<ExerciseData>)> =
        due_items::table.left_outer_join(exercise_data::table)
            .filter(due_items::user_id.eq(user_id))
            .order(due_items::due_date.asc())
            .first(&**conn)
            .optional()?;

    let due_item = due_questions.into_iter()
        .zip(due_exercises)
        .next()
        .map(|zipped| match zipped {
                 ((di, Some(qdata)), (_, None)) => (di, QuizType::Question(qdata.question_id)),
                 ((_, None), (di, Some(edata))) => (di, QuizType::Exercise(edata.exercise_id)),
                 e => {
            println!("WHY? {:?}", e);
            unreachable!()
        }
             });

    Ok(due_item)
}

pub fn count_overdue_items(conn: &Connection, user_id: i32) -> Result<i64> {
    use schema::due_items;

    let count: i64 = due_items::table.filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::offset::Utc::now()))
        .count()
        .get_result(&**conn)?;

    Ok(count)
}

fn choose_random_overdue_item(conn: &Connection, user_id: i32) -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due: Option<DueItem> = due_items::table.filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::offset::Utc::now()))
        .filter(due_items::cooldown_delay.lt(chrono::offset::Utc::now()))
        .order(sql::random)
        .first(&**conn)
        .optional()?;

    Ok(match due {
           Some(ref due) if due.item_type == "question" => {
            Some(QuizType::Question(question_data::table.filter(question_data::due.eq(due.id))
                .get_result::<QuestionData>(&**conn)?
                .question_id))
        }
           Some(ref due) if due.item_type == "exercise" => {
            Some(QuizType::Exercise(exercise_data::table.filter(exercise_data::due.eq(due.id))
                .get_result::<ExerciseData>(&**conn)?
                .exercise_id))
        }
           Some(_) => {
               return Err(ErrorKind::DatabaseOdd("Database contains due_item with an odd item_type \
                                               value!")
                                  .into())
           }
           None => None,
       })
}

fn choose_random_overdue_item_include_cooldown(conn: &Connection,
                                               user_id: i32)
                                               -> Result<Option<QuizType>> {
    use schema::{due_items, question_data, exercise_data};

    let due: Option<DueItem> = due_items::table.filter(due_items::user_id.eq(user_id))
        .filter(due_items::due_date.lt(chrono::offset::Utc::now()))
        .order(sql::random)
        .first(&**conn)
        .optional()?;

    Ok(match due {
           Some(ref due) if due.item_type == "question" => {
            Some(QuizType::Question(question_data::table.filter(question_data::due.eq(due.id))
                .get_result::<QuestionData>(&**conn)?
                .question_id))
        }
           Some(ref due) if due.item_type == "exercise" => {
            Some(QuizType::Exercise(exercise_data::table.filter(exercise_data::due.eq(due.id))
                .get_result::<ExerciseData>(&**conn)?
                .exercise_id))
        }
           Some(_) => {
               return Err(ErrorKind::DatabaseOdd("Database contains due_item with an odd item_type \
                                               value!")
                                  .into())
           }
           None => None,
       })
}

fn choose_new_question(conn: &Connection, user_id: i32) -> Result<Option<QuizQuestion>> {
    use diesel::expression::dsl::*;
    /*
    use schema::{quiz_questions, question_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(question_data::table)
        .select(question_data::question_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
            // Take only skills with level >= 2 (=both words introduced) before:
        .filter(skill_data::skill_level.ge(2))
        .filter(skill_data::user_id.eq(user_id));

    let new_question : Option<QuizQuestion> = quiz_questions::table
        .filter(quiz_questions::id.ne(all(dues)))
        .filter(quiz_questions::skill_id.eq(any(skills)))
        .filter(quiz_questions::published.eq(true))
        .order(sql::random)
        .first(&**conn)
        .optional()?;
*/
    let new_question: Option<QuizQuestion> =
        sql::<(
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Text,
        diesel::sql_types::Text,
        diesel::sql_types::Text,
        diesel::sql_types::Bool,
        diesel::sql_types::Integer,
        )>(&format!(r###"
SELECT
    id,
    skill_id,
    q_name,
    q_explanation,
    question_text,
    published,
    q.skill_level
FROM
    quiz_questions AS q
    LEFT OUTER JOIN
    (
        SELECT
            skill_level,
            skill_nugget
        FROM skill_data
        WHERE user_id={}
    ) AS s
    ON skill_nugget=skill_id
WHERE
    q.skill_level <= COALESCE(s.skill_level, 0) AND
    q.published = true AND
    q.id NOT IN ( SELECT question_id FROM due_items JOIN question_data ON id=due WHERE user_id={} )
ORDER BY RANDOM();
"###, user_id, user_id)) // Injection isn't possible: user_id is numerical and non-tainted data.
        .get_result(&**conn)
        .optional()?;

    Ok(new_question)
}

fn choose_new_exercise(conn: &Connection, user_id: i32) -> Result<Option<Exercise>> {
    use diesel::expression::dsl::*;
    /*
    use schema::{exercises, exercise_data, due_items, skill_data};
    let dues = due_items::table
        .inner_join(exercise_data::table)
        .select(exercise_data::exercise_id)
        .filter(due_items::user_id.eq(user_id));

    let skills = skill_data::table
        .select(skill_data::skill_nugget)
            // Take only skills with level >= 2 (=both words introduced) before:
        .filter(skill_data::skill_level.ge(2))
        .filter(skill_data::user_id.eq(user_id));

    let new_exercise : Option<Exercise> = exercises::table
        .filter(exercises::id.ne(all(dues)))
        .filter(exercises::skill_id.eq(any(skills)))
        .filter(exercises::published.eq(true))
        .order(sql::random)
        .first(&**conn)
        .optional()?;
*/
    let new_exercise: Option<Exercise> =
        sql::<(
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Bool,
        diesel::sql_types::Integer,
        )>(&format!(r###"
SELECT
    id,
    skill_id,
    published,
    e.skill_level
FROM
    exercises AS e
    LEFT OUTER JOIN
    (
        SELECT
            skill_level,
            skill_nugget
        FROM skill_data
        WHERE user_id={}
    ) AS s
    ON skill_nugget=skill_id
WHERE
    e.skill_level <= COALESCE(s.skill_level, 0) AND
    e.published = true AND
    e.id NOT IN ( SELECT exercise_id FROM due_items JOIN exercise_data ON id=due WHERE user_id={} )
ORDER BY RANDOM();
"###, user_id, user_id)) // Injection isn't possible: user_id is numerical and non-tainted data.
        .get_result(&**conn)
        .optional()?;

    Ok(new_exercise)
}

fn choose_cooldown_q_or_e(conn: &Connection,
                          user_id: i32,
                          metrics: &UserMetrics)
                          -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break ||
       metrics.quizes_today >= metrics.max_quizes_today ||
       metrics.break_until > chrono::offset::Utc::now() {
        return Ok(None);
    }

    if let Some(quiztype) = choose_random_overdue_item_include_cooldown(conn, user_id)? {
        return Ok(Some(quiztype));
    }

    Ok(None)
}

fn choose_new_q_or_e(conn: &Connection, user_id: i32) -> Result<Option<QuizType>> {

    if user::check_user_group(conn, user_id, "questions")? {
        if let Some(q) = choose_new_question(conn, user_id)? {
            return Ok(Some(QuizType::Question(q.id)));
        }
    }

    if user::check_user_group(conn, user_id, "exercises")? {
        if let Some(e) = choose_new_exercise(conn, user_id)? {
            return Ok(Some(QuizType::Exercise(e.id)));
        }
    }
    Ok(None)
}

fn choose_q_or_e(conn: &Connection,
                 user_id: i32,
                 metrics: &UserMetrics)
                 -> Result<Option<QuizType>> {

    if metrics.quizes_since_break >= metrics.max_quizes_since_break ||
       metrics.quizes_today >= metrics.max_quizes_today ||
       metrics.break_until > chrono::offset::Utc::now() {
        debug!("Enough quizes for today. The break/daily limits are full: Quizes/break: {} Quizes/day: {}.",
            metrics.quizes_since_break, metrics.quizes_today);
        return Ok(None);
    }

    debug!("Choosing random overdue item for user {}", user_id);
    if let Some(quiztype) = choose_random_overdue_item(conn, user_id)? {
        debug!("There is an overdue quiz item; presenting that.");
        return Ok(Some(quiztype));
    }

    debug!("Choosing new question for user {}", user_id);
    if let Some(quiztype) = choose_new_q_or_e(conn, user_id)? {
        debug!("No overdue quiz items; presenting a new quiz.");
        return Ok(Some(quiztype));
    }

    debug!("No overdue or new quiz items; returning.");

    Ok(None)
}

/// Choose a new word by random.
/// The word must be published, the required skill level of the word must be smaller
/// than the user's level on the corresponding skill and the word must not be seen before.
fn choose_new_random_word(conn: &Connection, user_id: i32) -> Result<Option<Word>> {
    use diesel::expression::dsl::*;

    /* // Diesel doesn't support this complex queries yet, so doing it directly in SQL
    use schema::{pending_items, words, w_asked_data, skill_data};
    let seen = pending_items::table
        .inner_join(w_asked_data::table)
        .select(w_asked_data::word_id)
        .filter(pending_items::user_id.eq(user_id));

    let new_word : Option<Word> = words::table
        .left_outer_join(skill_data::table)
        .filter(words::id.ne(all(seen)))
        .filter(words::published.eq(true))
        .order(sql::random)
        .first(&**conn)
        .optional()?;
*/
    let new_word: Option<Word> =
        sql::<(
        diesel::sql_types::Integer,
        diesel::sql_types::Text,
        diesel::sql_types::Text,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Bool,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        )>(&format!(r###"
SELECT
    id,
    word,
    explanation,
    audio_bundle,
    words.skill_nugget,
    published,
    words.skill_level,
    priority
FROM
    words
    LEFT OUTER JOIN
    (
        SELECT
            skill_level,
            skill_nugget
        FROM skill_data
        WHERE user_id={}
    ) AS skill_data
    ON skill_data.skill_nugget=words.skill_nugget
WHERE
    words.skill_level <= COALESCE(skill_data.skill_level, 0) AND
    words.published = true AND
    words.id NOT IN (
        SELECT word_id
            FROM pending_items
            JOIN w_asked_data
            ON pending_items.id=w_asked_data.id
            WHERE user_id={} AND pending_items.test_item=false
    )
ORDER BY words.priority DESC, RANDOM();
"###, user_id, user_id)) // Injection isn't possible: user_id is numerical and non-tainted data.
        .get_result(&**conn)
        .optional()?;

    Ok(new_word)
}

/// Choose a new word by random.
/// The word must be published, the required skill level of the word must be smaller
/// than the user's level on the corresponding skill and the word must not be seen before.
/// Additionally, the user's
/// skill level on the corresponding skill must be greater than zero.
/// This ensures that only words of skills that the user
/// has experience with are selected.
fn choose_new_paired_word(conn: &Connection, user_id: i32) -> Result<Option<Word>> {
    use diesel::expression::dsl::*;
    /*
    use schema::{pending_items, words, w_asked_data, skill_nuggets, skill_data};
    let seen = pending_items::table
        .inner_join(w_asked_data::table)
        .filter(pending_items::user_id.eq(user_id))
        .select(w_asked_data::word_id);

    let other_pair_seen = skill_nuggets::table
        .inner_join(skill_data::table)
        .filter(skill_data::user_id.eq(user_id))
        .filter(skill_data::skill_level.gt(0))
        .select(skill_nuggets::id);

    let new_word : Option<Word> = words::table
        .filter(words::id.ne(all(seen)))
        .filter(words::published.eq(true))
        .filter(words::skill_nugget.eq(any(other_pair_seen)))
        .order(sql::random)
        .first(&**conn)
        .optional()?;
*/
    let new_word: Option<Word> =
        sql::<(
        diesel::sql_types::Integer,
        diesel::sql_types::Text,
        diesel::sql_types::Text,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Bool,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        )>(&format!(r###"
SELECT
    id,
    word,
    explanation,
    audio_bundle,
    words.skill_nugget,
    published,
    words.skill_level,
    priority
FROM
    words
    LEFT OUTER JOIN
    (
        SELECT
            skill_level,
            skill_nugget
        FROM skill_data
        WHERE user_id={}
    ) AS skill_data
    ON skill_data.skill_nugget=words.skill_nugget
WHERE
    COALESCE(skill_data.skill_level, 0) > 0 AND
    words.skill_level <= COALESCE(skill_data.skill_level, 0) AND
    words.published = true AND
    words.id NOT IN (
        SELECT word_id
            FROM pending_items
            JOIN w_asked_data
            ON pending_items.id=w_asked_data.id
            WHERE user_id={} AND pending_items.test_item=false
    )
ORDER BY words.priority DESC, RANDOM();
"###, user_id, user_id)) // Injection isn't possible: user_id is numerical and non-tainted data.
        .get_result(&**conn)
        .optional()?;

    Ok(new_word)
}

fn choose_new_word(conn: &Connection, metrics: &mut UserMetrics) -> Result<Option<Word>> {


    if metrics.quizes_since_break >= metrics.max_quizes_since_break ||
       metrics.quizes_today >= metrics.max_quizes_today ||
       metrics.new_words_since_break >= metrics.max_words_since_break ||
       metrics.new_words_today >= metrics.max_words_today ||
       metrics.break_until > chrono::offset::Utc::now() {
        debug!("Enough words for today. The break/daily limits are full: Quizes/break: {} Quizes/day: {} Words/break: {} Words/day: {}.",
            metrics.quizes_since_break, metrics.quizes_today, metrics.new_words_since_break, metrics.new_words_today);
        return Ok(None);
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

fn clear_limits(conn: &Connection, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {
    use schema::user_stats;

    if chrono::offset::Utc::now() < metrics.break_until {
        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson {
                                   quiz_type: "future",
                                   due_date: due_string,
                               })));
    }

    // This is important because we have to zero the counts
    // once every day even though we wouldn't break a single time!
    // We are having 1 AM as the change point of the day
    // (That's 3 AM in Finland, where the users of this app are likely to reside)
    if chrono::offset::Utc::now() > (metrics.today.date().and_hms(1, 0, 0) + chrono::Duration::hours(24)) {

        let mut stats: UserStats = user_stats::table.filter(user_stats::id.eq(metrics.id))
            .get_result(&**conn)?;

        stats.days_used += 1;
        let _: UserStats = stats.save_changes(&**conn)?;

        metrics.today = chrono::offset::Utc::today().and_hms(1, 0, 0);
        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
    }
    Ok(None)
}

pub fn things_left_to_do(conn: &Connection,
                         user_id: i32)
                         -> Result<(Option<chrono::DateTime<chrono::offset::Utc>>, bool, bool)> {

    let next_existing_due =
        choose_next_due_item(conn, user_id)?.map(|(due_item, _)| due_item.due_date);
    let no_new_words = choose_new_random_word(conn, user_id)?.is_none();
    let no_new_quizes = choose_new_q_or_e(conn, user_id)?.is_none();

    Ok((next_existing_due, no_new_words, no_new_quizes))
}


fn check_break(conn: &Connection, user_id: i32, metrics: &mut UserMetrics) -> Result<Option<Quiz>> {
    use std::cmp::max;
    use chrono::Duration;

    let user_id = user_id;

    let (next_existing_due, no_new_words, no_new_quizes) = things_left_to_do(conn, user_id)?;

    debug!("No new words available: {:?}", no_new_words);
    debug!("No new quizes available: {:?} (either limited by lack of new words or by lack of \
            quizes themselves)",
           no_new_quizes);
    debug!("No due quizes available: {:?}", next_existing_due.is_none());

    if no_new_words && no_new_quizes && next_existing_due.is_none() {
        return Ok(None); // No words to study
    }

    let current_overdue = choose_random_overdue_item(conn, user_id)?;

    let words = metrics.new_words_today >= metrics.max_words_today || no_new_words;
    let new_quizes = metrics.quizes_today >= metrics.max_quizes_today || no_new_quizes;
    let due_quizes = metrics.quizes_today >= metrics.max_quizes_today || current_overdue.is_none();

    if words && new_quizes && due_quizes {
        debug!("Starting a long break because the daily limits are full: Quizes today: {}/{} No words: {} No new quizes: {} No due quizes: {}",
            metrics.quizes_today, metrics.max_quizes_today, words, new_quizes, due_quizes);

        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;
        metrics.new_words_today = 0;
        metrics.quizes_today = 0;
        metrics.break_until = metrics.today + Duration::hours(24);

        if no_new_words && no_new_quizes {
            // Nothing else left but due items – so no use breaking until there is some available
            metrics.break_until =
                max(metrics.break_until,
                    next_existing_due.unwrap_or_else(|| chrono::MAX_DATE.and_hms(0, 0, 0)));
        }

        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson {
                                   quiz_type: "future",
                                   due_date: due_string,
                               })));
    }

    let no_words = metrics.new_words_since_break >= metrics.max_words_since_break || no_new_words;
    let no_new_quizes = metrics.quizes_since_break >= metrics.max_quizes_since_break || no_new_quizes;
    let no_due_quizes = metrics.quizes_since_break >= metrics.max_quizes_since_break ||
                     current_overdue.is_none();

    // Start a break if the limits are full.
    // New: break is going to start even if word limit is not full, if the quiz limits are
    // this is because we don't want to introduce new words if users are overwhelmed with
    // quizes of already existing ones

    if no_words || (no_new_quizes && no_due_quizes) {
        debug!("Starting a short break because the break limits are full: Quizes since break: {}/{} No words: {} No new quizes: {} No due quizes: {}",
            metrics.quizes_since_break, metrics.max_quizes_since_break, no_words, no_new_quizes, no_due_quizes);

        let time_since_last_break = chrono::offset::Utc::now().signed_duration_since(metrics.break_until);

        let discounted_breaktime = max(Duration::seconds(0),
                                       Duration::seconds(metrics.break_length as i64) -
                                       time_since_last_break);

        metrics.break_until = chrono::offset::Utc::now() + discounted_breaktime;

        if no_new_words && no_new_quizes {
            // Nothing else left but due items – so no use breaking until there is some available
            metrics.break_until =
                max(metrics.break_until,
                    next_existing_due.unwrap_or_else(|| chrono::MAX_DATE.and_hms(0, 0, 0)));
        }

        metrics.new_words_since_break = 0;
        metrics.quizes_since_break = 0;

        let due_string = metrics.break_until.to_rfc3339();
        return Ok(Some(Quiz::F(FutureJson {
                                   quiz_type: "future",
                                   due_date: due_string,
                               })));
    }

    unreachable!("check_break. Either the break limits or daily limits should be full and those \
                  code paths return, so this should never happen. Words: {:?} New quizes: {:?} Due quizes: {:?}", no_words, no_new_quizes, no_due_quizes);
}


fn ask_new_question(conn: &Connection, id: i32) -> Result<(QuizQuestion, i32, Vec<Answer>, i32)> {
    let (question, answers, q_audio_bundles) = try_or!{ load_question(conn, id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "This function was called on the premise that the data exists!"
                    )) };

    let mut rng = thread_rng();
    let random_answer_index = rng.gen_range(0, answers.len());
    let right_answer_id = answers[random_answer_index].id;
    let q_audio_bundle = &q_audio_bundles[random_answer_index];

    let q_audio_file = audio::load_random_from_bundle(conn, q_audio_bundle.id)?;

    Ok((question, right_answer_id, answers, q_audio_file.id))
}

fn ask_new_exercise(conn: &Connection, id: i32) -> Result<(Exercise, Word, i32)> {
    let (exercise, _, mut words) = try_or!( load_exercise(conn, id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "This function was called on the premise that the data exists!"
                    )) );

    let random_answer_index = thread_rng().gen_range(0, words.len());
    let word = words.swap_remove(random_answer_index);

    let audio_file = audio::load_random_from_bundle(conn, word.audio_bundle)?;

    Ok((exercise, word, audio_file.id))
}

pub fn penditem_to_quiz(conn: &Connection, pi: &PendingItem) -> Result<Quiz> {
    use schema::{q_asked_data, e_asked_data, w_asked_data};

    Ok(match pi {
           pi if pi.item_type == "question" => {

        let asked: QAskedData = q_asked_data::table.filter(q_asked_data::id.eq(pi.id))
            .get_result(&**conn).chain_err(|| ErrorKind::DatabaseOdd("Bug: If item was in pending, it should also be in asked data?!"))?;

        let (question, answers, _) = try_or!{ load_question(conn, asked.question_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exist!"
                    )) };

        let mut answer_choices: Vec<_> =
            answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
            answer_choices.shuffle(&mut thread_rng());

        Quiz::Q(QuestionJson {
                    quiz_type: "question",
                    asked_id: pi.id,
                    explanation: question.q_explanation,
                    question: question.question_text,
                    right_a: asked.correct_qa_id,
                    answers: answer_choices,
                })

    }
           pi if pi.item_type == "exercise" => {

        let asked: EAskedData = e_asked_data::table.filter(e_asked_data::id.eq(pi.id))
            .get_result(&**conn)?;

        let word = try_or!{ load_word(conn, asked.word_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exists!"
                    )) };

        Quiz::E(ExerciseJson {
                    quiz_type: "exercise",
                    event_name: "training",
                    asked_id: pi.id,
                    word: word.word.nfc().collect::<String>(),
                    explanation: word.explanation,
                    must_record: false,
                })

    }
           pi if pi.item_type == "word" => {

        let asked: WAskedData = w_asked_data::table.filter(w_asked_data::id.eq(pi.id))
            .get_result(&**conn)?;

        let word = try_or!{ load_word(conn, asked.word_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exist!"
                    )) };

        Quiz::W(WordJson {
                    quiz_type: "word",
                    asked_id: pi.id,
                    word: word.word.nfc().collect::<String>(),
                    explanation: word.explanation,
                    show_accents: asked.show_accents,
                })
    }
           _ => unreachable!("Bug: There is only three kinds of quiz types!"),
       })
}

pub fn return_pending_item(conn: &Connection, user_id: i32) -> Result<Option<Quiz>> {
    use schema::pending_items;

    let pending_item: Option<PendingItem> =
        pending_items::table.filter(pending_items::user_id.eq(user_id))
            .filter(pending_items::pending.eq(true).and(pending_items::test_item.eq(false)))
            .get_result(&**conn)
            .optional()?;

    let quiz_type = match pending_item {
        Some(ref pi) => penditem_to_quiz(conn, pi)?,
        None => return Ok(None),
    };

    debug!("There was a pending item! Returning it.");

    Ok(Some(quiz_type))
}

pub fn return_q_or_e(conn: &Connection, user_id: i32, quiztype: QuizType) -> Result<Option<Quiz>> {

    match quiztype {
        QuizType::Question(id) => {

            let (question, right_a_id, answers, q_audio_id) = ask_new_question(conn, id)?;

            let pending_item =
                new_pending_item(conn, user_id, QuizType::Question(q_audio_id), false)?;

            let asked_data = QAskedData {
                id: pending_item.id,
                question_id: question.id,
                correct_qa_id: right_a_id,
            };

            register_future_q_answer(conn, &asked_data)?;

            let mut answer_choices: Vec<_> =
                answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
            answer_choices.shuffle(&mut thread_rng());

            let quiz_json = QuestionJson {
                quiz_type: "question",
                asked_id: pending_item.id,
                explanation: question.q_explanation,
                question: question.question_text,
                right_a: right_a_id,
                answers: answer_choices,
            };

            Ok(Some(Quiz::Q(quiz_json)))

        }
        QuizType::Exercise(id) => {

            let (exercise, word, audio_id) = ask_new_exercise(conn, id)?;

            let pending_item =
                new_pending_item(conn, user_id, QuizType::Exercise(audio_id), false)?;

            let asked_data = EAskedData {
                id: pending_item.id,
                exercise_id: exercise.id,
                word_id: word.id,
            };

            register_future_e_answer(conn, &asked_data)?;

            let quiz_json = ExerciseJson {
                quiz_type: "exercise",
                event_name: "training",
                asked_id: pending_item.id,
                word: word.word.nfc().collect::<String>(),
                explanation: word.explanation,
                must_record: false,
            };

            Ok(Some(Quiz::E(quiz_json)))
        }
        QuizType::Word(_) => unreachable!(),
    }
}

pub fn return_word(conn: &Connection, user_id: i32, the_word: Word) -> Result<Option<Quiz>> {

    let audio_file = audio::load_random_from_bundle(&*conn, the_word.audio_bundle)?;
    let show_accents = user::check_user_group(conn, user_id, "show_accents")?;

    let pending_item = new_pending_item(conn, user_id, QuizType::Word(audio_file.id), false)?;

    let asked_data = WAskedData {
        id: pending_item.id,
        word_id: the_word.id,
        show_accents: show_accents,
    };

    register_future_w_answer(conn, &asked_data)?;

    let quiz_json = WordJson {
        quiz_type: "word",
        word: the_word.word.nfc().collect::<String>(),
        explanation: the_word.explanation,
        asked_id: pending_item.id,
        show_accents: show_accents,
    };

    Ok(Some(Quiz::W(quiz_json)))
}

pub fn get_word_by_str(conn: &Connection, word: &str) -> Result<Word> {
    use schema::words;

    Ok(words::table.filter(words::word.eq(word)).get_result(&**conn)?)
}
pub fn get_word_by_id(conn: &Connection, id: i32) -> Result<Word> {
    use schema::words;

    Ok(words::table.filter(words::id.eq(id)).get_result(&**conn)?)
}

pub fn get_exercise(conn: &Connection, word: &str) -> Result<(Exercise, ExerciseVariant)> {
    use schema::{words, exercises, exercise_variants};

    let word: Word = words::table.filter(words::word.eq(word)).get_result(&**conn)?;

    Ok(exercises::table.inner_join(exercise_variants::table)
           .filter(exercise_variants::id.eq(word.id))
           .get_result(&**conn)?)
}

pub fn get_question(conn: &Connection, answer_text: &str) -> Result<(QuizQuestion, Answer)> {
    use schema::{quiz_questions, question_answers, audio_bundles};

    let bundle: AudioBundle = audio_bundles::table.filter(audio_bundles::listname.eq(answer_text))
        .get_result(&**conn)?;

    Ok(quiz_questions::table.inner_join(question_answers::table)
           .filter(question_answers::q_audio_bundle.eq(bundle.id))
           .get_result(&**conn)?)
}

pub enum QuizSerialized {
    Word(&'static str, i32),
    Question(&'static str, i32),
    Exercise(&'static str, i32),
}

impl QuizSerialized {
    pub fn from_iter<I: Iterator<Item=&'static str>>(mut iter: I) -> Result<Self> {
        let col_err = || Error::from("Should have three columns.");
        let variant = iter.next().ok_or_else(col_err)?;
        let word = iter.next().ok_or_else(col_err)?;
        let id: i32 = iter.next().ok_or_else(col_err)?.parse()?;
        Ok(match variant {
            "word" => QuizSerialized::Word(word, id),
            "question" => QuizSerialized::Question(word, id),
            "exercise" => QuizSerialized::Exercise(word, id),
            _ => bail!("Expected word/question/exercise"),
        })
    }
}

pub fn read_quiz_tsv(tsv: String) -> Result<Vec<QuizSerialized>> {
    let mut quiz = Vec::new();
    for line in Box::leak(tsv.into_boxed_str()).lines() {
        if line.starts_with("//") || line.trim().len() == 0 { continue }
        quiz.push(QuizSerialized::from_iter(line.split('\t'))?);
    }
    Ok(quiz)
}

pub fn test_item(conn: &Connection,
                 user_id: i32,
                 quiz_str: &QuizSerialized)
                 -> Result<(Quiz, i32)> {
    let pending_item;
    let test_item = match *quiz_str {
        QuizSerialized::Word(s, audio_id) => {
            let w = quiz::get_word_by_str(conn, s).chain_err(|| format!("Word {} not found", s))?;
            let a = audio::get_audio_file_by_id(conn, audio_id)?;

            if w.audio_bundle != a.bundle_id {
                let bundle = audio::get_bundle_by_id(conn, a.bundle_id);
                panic!("Word: {:?}.\nAudio bundle: {:?}", w, bundle);
            }

            pending_item = new_pending_item(conn, user_id, QuizType::Word(audio_id), true)?;

            let asked_data = WAskedData {
                id: pending_item.id,
                word_id: w.id,
                show_accents: false,
            };

            register_future_w_answer(conn, &asked_data)?;

            let word = try_or!{ load_word(conn, asked_data.word_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exists!"
                    )) };

            Quiz::W(WordJson {
                        quiz_type: "word",
                        asked_id: pending_item.id,
                        word: word.word.nfc().collect::<String>(),
                        explanation: word.explanation,
                        show_accents: asked_data.show_accents,
                    })
        }
        QuizSerialized::Question(s, audio_id) => {

            let (q, ans) =
                quiz::get_question(conn, s).chain_err(|| format!("Question {} not found", s))?;

            let a = audio::get_audio_file_by_id(conn, audio_id)?;

            if ans.q_audio_bundle != a.bundle_id {
                let bundle = audio::get_bundle_by_id(conn, a.bundle_id);
                panic!("Q Answer: {:?}.\nAudio bundle: {:?}", ans, bundle);
            }

            pending_item = new_pending_item(conn, user_id, QuizType::Question(audio_id), true)?;

            let asked_data = QAskedData {
                id: pending_item.id,
                question_id: q.id,
                correct_qa_id: ans.id,
            };

            register_future_q_answer(conn, &asked_data)?;

            let (question, answers, _) = try_or!{ load_question(conn, asked_data.question_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exists!"
                    )) };

            let mut answer_choices: Vec<_> =
                answers.into_iter().map(|a| (a.id, a.answer_text)).collect();
                answer_choices.shuffle(&mut thread_rng());


            Quiz::Q(QuestionJson {
                        quiz_type: "question",
                        asked_id: pending_item.id,
                        explanation: question.q_explanation,
                        question: question.question_text,
                        right_a: asked_data.correct_qa_id,
                        answers: answer_choices,
                    })

        }
        QuizSerialized::Exercise(word, audio_id) => {

            let (e, var) = quiz::get_exercise(conn, word)
                        .chain_err(|| format!("Exercise {} not found", word))?;

            let w = quiz::get_word_by_id(conn, var.id)?;
            let a = audio::get_audio_file_by_id(conn, audio_id)?;

            if w.audio_bundle != a.bundle_id {
                let bundle = audio::get_bundle_by_id(conn, a.bundle_id);
                panic!("Word: {:?}.\nAudio bundle: {:?}", w, bundle);
            }

            pending_item = new_pending_item(conn, user_id, QuizType::Exercise(audio_id), true)?;

            let asked_data = EAskedData {
                id: pending_item.id,
                exercise_id: e.id,
                word_id: var.id,
            };

            register_future_e_answer(conn, &asked_data)?;

            let word = try_or!{ load_word(conn, asked_data.word_id)?,
                else bail!(
                    ErrorKind::DatabaseOdd(
                        "Bug: If the item was set pending in the first place, it should exists!"
                    )) };

            Quiz::E(ExerciseJson {
                        quiz_type: "exercise",
                        event_name: "pretest or posttest",
                        asked_id: pending_item.id,
                        word: word.word.nfc().collect::<String>(),
                        explanation: word.explanation,
                        must_record: false,
                    })
        }
    };

    debug!("There was a test item! Returning it.");

    Ok((test_item, pending_item.id))
}



/* MAIN LOGIC */

fn get_new_quiz_inner(conn: &Connection,
                      user_id: i32,
                      metrics: &mut UserMetrics)
                      -> Result<Option<Quiz>> {
    debug!("Get new quiz for user {}", user_id);

    // Pending item first (items that were asked,
    // but not answered because of loss of connection, user closing the session etc.)

    if let Some(pending_quiz) = return_pending_item(conn, user_id)? {
        return Ok(Some(pending_quiz));
    }

    // Clear the per-day limits if it's tomorrow already and stop if we are in the middle of a break
    if let Some(future) = clear_limits(conn, metrics)? {
        return Ok(Some(future));
    }

    // After that, question & exercise reviews that are overdue
    // (except if they are on a cooldown period), and after that, new ones

    if let Some(quiztype) = choose_q_or_e(conn, user_id, metrics)? {
        metrics.quizes_today += 1;
        metrics.quizes_since_break += 1;
        return return_q_or_e(conn, user_id, quiztype);
    }


    // No questions & exercises available at the moment. Introducing new words

    if let Some(the_word) = choose_new_word(conn, metrics)? {
        metrics.new_words_today += 1;
        metrics.new_words_since_break += 1;
        return return_word(conn, user_id, the_word);
    }


    // If there's nothing else, time to show the cooled-down stuff!

    if let Some(quiztype) = choose_cooldown_q_or_e(conn, user_id, metrics)? {
        metrics.quizes_today += 1;
        metrics.quizes_since_break += 1;
        return return_q_or_e(conn, user_id, quiztype);
    }

    // There seems to be nothig to do?!
    // Either there is no more words to study or the limits are full.

    if let Some(future) = check_break(conn, user_id, metrics)? {
        return Ok(Some(future));
    }

    Ok(None) // No words left to study
}







/* PUBLIC APIS */

pub fn get_new_quiz(conn: &Connection, user_id: i32) -> Result<Option<Quiz>> {
    use schema::user_metrics;

    let mut metrics: UserMetrics = user_metrics::table.filter(user_metrics::id.eq(user_id))
        .get_result(&**conn)?;

    let result = get_new_quiz_inner(conn, user_id, &mut metrics)?;

    let _: UserMetrics = metrics.save_changes(&**conn)?;

    Ok(result)
}


pub fn get_next_quiz(conn: &Connection,
                     user_id: i32,
                     answer_enum: Answered)
                     -> Result<Option<Quiz>> {
    use schema::user_metrics;

    let mut metrics: UserMetrics = user_metrics::table.filter(user_metrics::id.eq(user_id))
        .get_result(&**conn)?;

    match answer_enum {
        Answered::W(answer_word) => {
            log_answer_word(conn, user_id, &answer_word)?;
        }
        Answered::E(exercise) => {
            log_answer_exercise(conn, user_id, &exercise, &metrics)?;
        }
        Answered::Q(answer) => {
            log_answer_question(conn, user_id, &answer, &metrics)?;
        }
    }

    let result = get_new_quiz_inner(conn, user_id, &mut metrics)?;

    let _: UserMetrics = metrics.save_changes(&**conn)?;

    Ok(result)
}
