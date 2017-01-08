use super::schema::*;
use chrono::{DateTime, UTC};
use try_map::FallibleMapExt;

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[has_many(passwords, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(user_metrics, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(user_stats, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(sessions, foreign_key = "user_id")]
#[has_many(skill_data, foreign_key = "user_id")]
#[has_many(event_experiences, foreign_key = "user_id")]
#[has_many(group_memberships, foreign_key = "user_id")]
#[has_many(anon_aliases, foreign_key = "user_id")]
#[has_many(pending_items, foreign_key = "user_id")]
#[has_many(due_items, foreign_key = "user_id")]
#[has_many(reset_email_secrets, foreign_key = "user_id")]
#[derive(Identifiable, Clone, Queryable, Debug, Associations, AsChangeset, RustcEncodable)]
pub struct User {
    pub id: i32,
    pub email: Option<String>,
    pub joined: DateTime<UTC>,
}


#[derive(Identifiable, Queryable, Debug, Insertable, Associations, AsChangeset)]
#[belongs_to(User, foreign_key = "id")]
#[table_name="passwords"]
pub struct Password {
    pub id: i32,
    pub password_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub initial_rounds: i16,
    pub extra_rounds: i16,
}

#[derive(Debug, Insertable)]
#[table_name="sessions"]
pub struct NewSession<'a> {
    pub proposed_token: &'a [u8],
    pub current_token: &'a [u8],
    pub retired_token: &'a [u8],
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: &'a [u8],
}



#[derive(Identifiable, Queryable, Debug, Associations, AsChangeset)]
#[table_name="sessions"]
#[belongs_to(User, foreign_key = "user_id")]
#[changeset_options(treat_none_as_null = "true")]
pub struct Session {
    pub id: i32,
    pub proposed_token: Vec<u8>,
    pub current_token: Vec<u8>,
    pub retired_token: Vec<u8>,
    pub access_version: i32,
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: Vec<u8>,
}


#[derive(Queryable, Debug, RustcEncodable, RustcDecodable)]
pub struct PendingEmailConfirm {
    pub secret: String,
    pub email: String,
    pub groups: Vec<i32>,
    pub added: DateTime<UTC>,
}

#[derive(Insertable)]
#[table_name="pending_email_confirms"]
pub struct NewPendingEmailConfirm<'a> {
    pub secret: &'a str,
    pub email: &'a str,
    pub groups: &'a [i32],
}

#[derive(Identifiable, Queryable, Debug, Insertable, Associations, AsChangeset, RustcEncodable, RustcDecodable)]
#[table_name="user_groups"]
#[has_many(group_memberships, foreign_key = "group_id")]
#[has_many(anon_aliases, foreign_key = "group_id")]
#[has_many(events, foreign_key = "required_group")]
pub struct UserGroup {
    pub id: i32,
    pub group_name: String,
    pub anonymous: bool,
}

#[derive(Identifiable, Queryable, Debug, Insertable, Associations, AsChangeset, RustcEncodable, RustcDecodable)]
#[table_name="group_memberships"]
#[primary_key(user_id, group_id)]
#[belongs_to(UserGroup, foreign_key = "group_id")]
#[belongs_to(User, foreign_key = "user_id")]
pub struct GroupMembership {
    pub user_id: i32,
    pub group_id: i32,
    pub anonymous: bool,
}

#[derive(Queryable, Debug, Insertable, Associations, AsChangeset)]
#[table_name="anon_aliases"]
#[belongs_to(UserGroup, foreign_key = "group_id")]
#[belongs_to(User, foreign_key = "user_id")]
pub struct AnonAliases {
    pub id: i32,
    pub name: String,
    pub user_id: Option<i32>,
    pub group_id: Option<i32>,
}

#[derive(Insertable)]
#[table_name="skill_nuggets"]
pub struct NewSkillNugget<'a> {
    pub skill_summary: &'a str,
}

#[derive(Insertable, Queryable, Associations, AsChangeset, Identifiable, Debug, RustcEncodable)]
#[table_name="skill_nuggets"]
#[has_many(quiz_questions, foreign_key = "skill_id")]
#[has_many(exercises, foreign_key = "skill_id")]
#[has_many(skill_data, foreign_key = "skill_nugget")]
#[has_many(words, foreign_key = "skill_nugget")]
pub struct SkillNugget {
    pub id: i32,
    pub skill_summary: String,
}

#[derive(Insertable)]
#[table_name="narrators"]
pub struct NewNarrator<'a> {
    pub name: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, AsChangeset, Debug, RustcEncodable, RustcDecodable)]
#[table_name="narrators"]
#[has_many(audio_files, foreign_key = "narrators_id")]
pub struct Narrator {
    pub id: i32,
    pub name: String,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name="audio_files"]
pub struct NewAudioFile<'a> {
    pub narrators_id: i32,
    pub bundle_id: i32,
    pub file_path: &'a str,
    pub mime: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable)]
#[table_name="audio_files"]
#[belongs_to(Narrator, foreign_key = "narrators_id")]
#[belongs_to(AudioBundle, foreign_key = "bundle_id")]
#[has_many(pending_items, foreign_key = "audio_file_id")]
pub struct AudioFile {
    pub id: i32,
    pub narrators_id: i32,
    pub bundle_id: i32,
    pub file_path: String,
    pub mime: String,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, AsChangeset, RustcEncodable, RustcDecodable)]
#[table_name="audio_bundles"]
#[has_many(audio_files, foreign_key = "bundle_id")]
#[has_many(question_answers, foreign_key = "q_audio_bundle")]
#[has_many(words, foreign_key = "audio_bundle")]
pub struct AudioBundle {
    pub id: i32,
    pub listname: String,
}

#[derive(Insertable)]
#[table_name="audio_bundles"]
pub struct NewAudioBundle<'a> {
    pub listname: &'a str,
}

#[derive(Insertable, Debug)]
#[table_name="quiz_questions"]
pub struct NewQuizQuestion<'a> {
    pub skill_id: i32,
    pub q_name: &'a str,
    pub q_explanation: &'a str,
    pub question_text: &'a str,
    pub skill_level: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable)]
#[belongs_to(SkillNugget, foreign_key = "skill_id")]
#[has_many(question_answers, foreign_key = "question_id")]
#[has_many(question_data, foreign_key = "question_id")]
#[has_many(q_asked_data, foreign_key = "question_id")]
#[table_name="quiz_questions"]
pub struct QuizQuestion {
    pub id: i32,
    pub skill_id: i32,
    pub q_name: String,
    pub q_explanation: String,
    pub question_text: String,
    pub published: bool,
    pub skill_level: i32,
}

#[derive(Queryable, AsChangeset, Debug, RustcEncodable, RustcDecodable)]
#[table_name="quiz_questions"]
pub struct UpdateQuestion {
    pub skill_id: Option<i32>,
    pub q_name: Option<String>,
    pub q_explanation: Option<String>,
    pub question_text: Option<String>,
    pub published: Option<bool>,
    pub skill_level: Option<i32>,
}

#[derive(Insertable, Debug)]
#[table_name="question_answers"]
pub struct NewAnswer<'a> {
    pub question_id: i32,
    pub a_audio_bundle: Option<i32>,
    pub q_audio_bundle: i32,
    pub answer_text: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable, AsChangeset)]
#[belongs_to(QuizQuestion, foreign_key = "question_id")]
#[belongs_to(AudioBundle, foreign_key = "q_audio_bundle")]
#[table_name="question_answers"]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub a_audio_bundle: Option<i32>,
    pub q_audio_bundle: i32,
    pub answer_text: String,
}


#[derive(Queryable, AsChangeset, Debug, RustcEncodable, RustcDecodable)]
#[table_name="question_answers"]
pub struct UpdateAnswer {
    pub question_id: Option<i32>,
    pub a_audio_bundle: Option<Option<i32>>,
    pub q_audio_bundle: Option<i32>,
    pub answer_text: Option<String>,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable, RustcDecodable)]
#[belongs_to(SkillNugget, foreign_key = "skill_id")]
#[has_many(exercise_variants, foreign_key = "exercise_id")]
#[has_many(exercise_data, foreign_key = "exercise_id")]
#[has_many(e_asked_data, foreign_key = "exercise_id")]
#[table_name="exercises"]
pub struct Exercise {
    pub id: i32,
    pub skill_id: i32,
    pub published: bool,
    pub skill_level: i32,
}

#[derive(Queryable, Debug, AsChangeset, RustcEncodable, RustcDecodable)]
#[table_name="exercises"]
pub struct UpdateExercise {
    pub skill_id: Option<i32>,
    pub published: Option<bool>,
    pub skill_level: Option<i32>,
}

#[derive(Insertable)]
#[table_name="exercises"]
pub struct NewExercise {
    pub skill_id: i32,
    pub skill_level: i32,
}

#[derive(Insertable, Identifiable, Queryable, Associations, Debug, RustcEncodable, RustcDecodable)]
#[belongs_to(Exercise, foreign_key = "exercise_id")]
#[belongs_to(Word, foreign_key = "id")]
#[table_name="exercise_variants"]
pub struct ExerciseVariant {
    pub id: i32,
    pub exercise_id: i32,
}

#[derive(Queryable, Debug, AsChangeset, RustcEncodable, RustcDecodable)]
#[table_name="exercise_variants"]
pub struct UpdateExerciseVariant {
    pub id: Option<i32>,
    pub exercise_id: Option<i32>,
}

#[derive(Insertable)]
#[table_name="words"]
pub struct NewWord<'a> {
    pub word: &'a str,
    pub explanation: &'a str,
    pub audio_bundle: i32,
    pub skill_nugget: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable, AsChangeset)]
#[table_name="words"]
#[belongs_to(SkillNugget, foreign_key = "skill_nugget")]
#[belongs_to(AudioBundle, foreign_key = "audio_bundle")]
#[has_many(w_asked_data, foreign_key = "word_id")]
#[has_many(exercise_variants, foreign_key = "id")]
pub struct Word {
    pub id: i32,
    pub word: String,
    pub explanation: String,
    pub audio_bundle: i32,
    pub skill_nugget: i32,
    pub published: bool,
}

#[derive(Queryable, AsChangeset, Debug, RustcEncodable, RustcDecodable)]
#[table_name="words"]
pub struct UpdateWord {
    pub word: Option<String>,
    pub explanation: Option<String>,
    pub audio_bundle: Option<i32>,
    pub skill_nugget: Option<i32>,
    pub published: Option<bool>,
}

#[derive(Insertable, Identifiable, Queryable, Associations, Debug, AsChangeset)]
#[table_name="due_items"]
#[belongs_to(User, foreign_key = "user_id")]
#[has_many(question_data, foreign_key = "due")]
#[has_many(exercise_data, foreign_key = "due")]
pub struct DueItem {
    pub id: i32,
    pub user_id: i32,
    pub due_date: DateTime<UTC>,
    pub due_delay: i32,
    pub cooldown_delay: DateTime<UTC>,
    pub correct_streak_overall: i32,
    pub correct_streak_this_time: i32,
    pub item_type: String,
}

#[derive(Insertable)]
#[table_name="due_items"]
pub struct NewDueItem<'a> {
    pub user_id: i32,
    pub due_date: DateTime<UTC>,
    pub due_delay: i32,
    pub cooldown_delay: DateTime<UTC>,
    pub correct_streak_overall: i32,
    pub correct_streak_this_time: i32,
    pub item_type: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, AsChangeset)]
#[table_name="pending_items"]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(AudioFile, foreign_key = "audio_file_id")]
#[has_many(q_asked_data, foreign_key = "id")]
#[has_many(e_asked_data, foreign_key = "id")]
#[has_many(w_asked_data, foreign_key = "id")]
pub struct PendingItem {
    pub id: i32,
    pub user_id: i32,
    pub audio_file_id: i32,
    pub asked_date: DateTime<UTC>,
    pub pending: bool,
    pub item_type: String,
}

#[derive(Insertable, Associations, Debug, AsChangeset)]
#[table_name="pending_items"]
pub struct NewPendingItem<'a> {
    pub user_id: i32,
    pub audio_file_id: i32,
    pub item_type: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, AsChangeset)]
#[table_name="q_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(QuizQuestion, foreign_key = "question_id")]
#[belongs_to(Answer, foreign_key = "correct_qa_id")]
#[has_many(q_answered_data, foreign_key = "id")]
pub struct QAskedData {
    pub id: i32,
    pub question_id: i32,
    pub correct_qa_id: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, RustcEncodable)]
#[table_name="q_answered_data"]
#[belongs_to(QAskedData, foreign_key = "id")]
pub struct QAnsweredData {
    pub id: i32,
    pub answered_qa_id: Option<i32>,
    pub answered_date: DateTime<UTC>,
    pub active_answer_time_ms: i32,
    pub full_answer_time_ms: i32,
    pub full_spent_time_ms: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, AsChangeset)]
#[table_name="e_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(Exercise, foreign_key = "exercise_id")]
#[belongs_to(Word, foreign_key = "word_id")]
#[has_many(e_answered_data, foreign_key = "id")]
pub struct EAskedData {
    pub id: i32,
    pub exercise_id: i32,
    pub word_id: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, AsChangeset, RustcEncodable)]
#[table_name="e_answered_data"]
#[belongs_to(EAskedData, foreign_key = "id")]
pub struct EAnsweredData {
    pub id: i32,
    pub answered_date: DateTime<UTC>,
    pub active_answer_time_ms: i32,
    pub full_answer_time_ms: i32,
    pub audio_times: i32,
    pub answer_level: i32,
    pub full_spent_time_ms: i32,
    pub reflected_time_ms: i32,
}

#[derive(Identifiable, Insertable, Queryable, Associations, Debug, Clone, AsChangeset)]
#[table_name="w_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(Word, foreign_key = "word_id")]
#[has_many(w_answered_data, foreign_key = "id")]
pub struct WAskedData {
    pub id: i32,
    pub word_id: i32,
    pub show_accents: bool,
}

#[derive(Identifiable, Insertable, Queryable, Associations, Debug, Clone, AsChangeset, RustcEncodable)]
#[table_name="w_answered_data"]
#[belongs_to(WAskedData, foreign_key = "id")]
pub struct WAnsweredData {
    pub id: i32,
    pub full_spent_time_ms: i32,
    pub audio_times: i32,
    pub checked_date: DateTime<UTC>,
    pub active_answer_time_ms: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset)]
#[table_name="question_data"]
#[belongs_to(DueItem, foreign_key = "due")]
pub struct QuestionData {
    pub question_id: i32,
    pub due: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset)]
#[table_name="exercise_data"]
#[belongs_to(DueItem, foreign_key = "due")]
#[belongs_to(Exercise, foreign_key = "exercise_id")]
pub struct ExerciseData {
    pub exercise_id: i32,
    pub due: i32,
}

#[derive(Insertable)]
#[table_name="skill_data"]
pub struct NewSkillData {
    pub user_id: i32,
    pub skill_nugget: i32,
    pub skill_level: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset)]
#[table_name="skill_data"]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(SkillNugget, foreign_key = "skill_nugget")]
pub struct SkillData {
    pub user_id: i32,
    pub skill_nugget: i32,
    pub skill_level: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset, Identifiable)]
#[belongs_to(User, foreign_key = "id")]
#[table_name="user_metrics"]
pub struct UserMetrics {
    pub id: i32,
    pub new_words_since_break: i32,
    pub new_words_today: i32,
    pub quizes_since_break: i32,
    pub quizes_today: i32,
    pub break_until:  DateTime<UTC>,
    pub today:  DateTime<UTC>,
    pub max_words_since_break: i32,
    pub max_words_today: i32,
    pub max_quizes_since_break: i32,
    pub max_quizes_today: i32,

    pub break_length: i32,
    pub delay_multiplier: i32,
    pub initial_delay: i32,
    pub streak_limit: i32,
    pub cooldown_delay: i32,
}

use rustc_serialize::{Encoder, Encodable, Decoder, Decodable};

impl Encodable for UserMetrics {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("UserMetrics", 16, |s| {
            s.emit_struct_field("id", 0, |s| { s.emit_i32(self.id) })?;
            s.emit_struct_field("new_words_since_break", 1, |s| { s.emit_i32(self.new_words_since_break) })?;
            s.emit_struct_field("new_words_today", 2, |s| { s.emit_i32(self.new_words_today) })?;
            s.emit_struct_field("quizes_since_break", 3, |s| { s.emit_i32(self.quizes_since_break) })?;
            s.emit_struct_field("quizes_today", 4, |s| { s.emit_i32(self.quizes_today) })?;
            s.emit_struct_field("break_until", 5, |s| { s.emit_str(&self.break_until.to_rfc3339()) })?;
            s.emit_struct_field("today", 6, |s| { s.emit_str(&self.today.to_rfc3339()) })?;
            s.emit_struct_field("max_words_since_break", 7, |s| { s.emit_i32(self.max_words_since_break) })?;
            s.emit_struct_field("max_words_today", 8, |s| { s.emit_i32(self.max_words_today) })?;
            s.emit_struct_field("max_quizes_since_break", 9, |s| { s.emit_i32(self.max_quizes_since_break) })?;
            s.emit_struct_field("max_quizes_today", 10, |s| { s.emit_i32(self.max_quizes_today) })?;
            s.emit_struct_field("break_length", 11, |s| { s.emit_i32(self.break_length) })?;
            s.emit_struct_field("delay_multiplier", 12, |s| { s.emit_i32(self.delay_multiplier) })?;
            s.emit_struct_field("initial_delay", 13, |s| { s.emit_i32(self.initial_delay) })?;
            s.emit_struct_field("streak_limit", 14, |s| { s.emit_i32(self.streak_limit) })?;
            s.emit_struct_field("cooldown_delay", 15, |s| { s.emit_i32(self.cooldown_delay) })?;
            Ok(())
        })
    }
}

#[derive(Debug, AsChangeset, Identifiable)]
#[table_name="user_metrics"]
pub struct UpdateUserMetrics {
    pub id: i32,
    pub new_words_since_break: Option<i32>,
    pub new_words_today: Option<i32>,
    pub quizes_since_break: Option<i32>,
    pub quizes_today: Option<i32>,
    pub break_until:  Option<DateTime<UTC>>,
    pub today:  Option<DateTime<UTC>>,
    pub max_words_since_break: Option<i32>,
    pub max_words_today: Option<i32>,
    pub max_quizes_since_break: Option<i32>,
    pub max_quizes_today: Option<i32>,

    pub break_length: Option<i32>,
    pub delay_multiplier: Option<i32>,
    pub initial_delay: Option<i32>,
    pub streak_limit: Option<i32>,
    pub cooldown_delay: Option<i32>,
}

impl Decodable for UpdateUserMetrics {
    fn decode<D: Decoder>(d: &mut D) -> Result<UpdateUserMetrics, D::Error> {
        d.read_struct("UserMetrics", 16, |d| {
            Ok(UpdateUserMetrics{
                id: d.read_struct_field("id", 0, Decodable::decode)?,
                new_words_since_break: d.read_struct_field("new_words_since_break", 1, Decodable::decode)?,
                new_words_today: d.read_struct_field("new_words_today", 2, Decodable::decode)?,
                quizes_since_break: d.read_struct_field("quizes_since_break", 3, Decodable::decode)?,
                quizes_today: d.read_struct_field("quizes_today", 4, Decodable::decode)?,
                break_until: d.read_struct_field("break_until", 5, <Option<String> as Decodable>::decode)?
                    .try_map(|s| DateTime::parse_from_rfc3339(&s).map_err(|_| d.error("Can't decode a date from rfc3339!")))?.map(|t| t.with_timezone(&UTC)),
                today: d.read_struct_field("today", 6, <Option<String> as Decodable>::decode)?
                    .try_map(|s| DateTime::parse_from_rfc3339(&s).map_err(|_| d.error("Can't decode a date from rfc3339!")))?.map(|t| t.with_timezone(&UTC)),
                max_words_since_break: d.read_struct_field("max_words_since_break", 7, Decodable::decode)?,
                max_words_today: d.read_struct_field("max_words_today", 8, Decodable::decode)?,
                max_quizes_since_break: d.read_struct_field("max_quizes_since_break", 9, Decodable::decode)?,
                max_quizes_today: d.read_struct_field("max_quizes_today", 10, Decodable::decode)?,
                break_length: d.read_struct_field("break_length", 11, Decodable::decode)?,
                delay_multiplier: d.read_struct_field("delay_multiplier", 12, Decodable::decode)?,
                initial_delay: d.read_struct_field("initial_delay", 13, Decodable::decode)?,
                streak_limit: d.read_struct_field("streak_limit", 14, Decodable::decode)?,
                cooldown_delay: d.read_struct_field("cooldown_delay", 15, Decodable::decode)?,
            })
        })
    }
}

#[derive(Insertable)]
#[table_name="user_metrics"]
pub struct NewUserMetrics {
    pub id: i32,
}

#[derive(Insertable)]
#[table_name="user_stats"]
pub struct NewUserStats {
    pub id: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset, Identifiable, RustcEncodable, RustcDecodable)]
#[belongs_to(User, foreign_key = "id")]
#[table_name="user_stats"]
pub struct UserStats {
    pub id: i32,
    pub days_used: i32,
    pub all_active_time_ms: i64,
    pub all_spent_time_ms: i64,
    pub all_words: i32,
    pub quiz_all_times: i32,
    pub quiz_correct_times: i32,
    pub last_nag_email: Option<DateTime<UTC>>,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, RustcEncodable, RustcDecodable)]
#[has_many(event_experiences, foreign_key = "event_id")]
#[belongs_to(UserGroup, foreign_key = "required_group")]
#[table_name="events"]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub published: bool,
    pub required_group: Option<i32>,
    pub priority: i32,
}

#[derive(Queryable, Identifiable, RustcEncodable, Debug, AsChangeset)]
#[table_name="events"]
pub struct UpdateEvent<'a> {
    pub id: i32,
    pub name: Option<&'a str>,
    pub published: Option<bool>,
    pub required_group: Option<Option<i32>>,
    pub priority: Option<i32>,
}

#[derive(Insertable)]
#[table_name="events"]
pub struct NewEvent<'a> {
    pub name: &'a str,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset, RustcEncodable)]
#[table_name="event_experiences"]
#[changeset_options(treat_none_as_null = "true")]
#[belongs_to(Event, foreign_key = "event_id")]
#[belongs_to(User, foreign_key = "user_id")]
pub struct EventExperience {
    pub user_id: i32,
    pub event_id: i32,
    pub event_init: DateTime<UTC>,
    pub event_finish: Option<DateTime<UTC>>,
}

#[derive(Insertable)]
#[table_name="event_experiences"]
pub struct NewEventExperience {
    pub user_id: i32,
    pub event_id: i32,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Debug, AsChangeset, RustcEncodable)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(Event, foreign_key = "event_id")]
#[changeset_options(treat_none_as_null = "true")]
#[table_name="event_userdata"]
pub struct EventUserdata {
    pub id: i32,
    pub user_id: i32,
    pub event_id: i32,
    pub created: DateTime<UTC>,
    pub key: Option<String>,
    pub data: String,
}

#[derive(Insertable)]
#[table_name="event_userdata"]
pub struct NewEventUserdata<'a> {
    pub user_id: i32,
    pub event_id: i32,
    pub key: Option<&'a str>,
    pub data: &'a str,
}

#[derive(AsChangeset)]
#[table_name="event_userdata"]
pub struct UpdateEventUserdata<'a> {
    pub data: &'a str,
}

#[derive(Queryable, Insertable, Debug, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name="reset_email_secrets"]
pub struct ResetEmailSecrets {
    pub user_id: i32,
    pub email: String,
    pub secret: String,
    pub added: DateTime<UTC>,
}
