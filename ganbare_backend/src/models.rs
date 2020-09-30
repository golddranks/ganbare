use super::schema::*;
use chrono::{DateTime, offset::Utc};
use serde::{Deserializer, Deserialize};
use serde::de::Visitor;
use std::marker::PhantomData;
use std::fmt::{self, Formatter};
use serde::de::Error;

pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    Deserialize::deserialize(de).map(Some)
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}


// actually, the relationship is one-to-1..0

// actually, the relationship is one-to-1..0

// actually, the relationship is one-to-1..0








#[derive(Identifiable, Clone, Queryable, Debug, Associations, AsChangeset, Serialize)]
pub struct User {
    pub id: i32,
    pub email: Option<String>,
    pub joined: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
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
    pub user_id: i32,
    pub started: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub secret: &'a [u8],
}


#[derive(Identifiable, Queryable, Debug, Associations, AsChangeset)]
#[table_name="sessions"]
#[belongs_to(User, foreign_key = "user_id")]
#[changeset_options(treat_none_as_null = "true")]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub started: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub secret: Vec<u8>,
    pub refresh_count: i32,
}


#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct PendingEmailConfirm {
    pub secret: String,
    pub email: String,
    pub groups: Vec<i32>,
    pub added: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name="pending_email_confirms"]
pub struct NewPendingEmailConfirm<'a> {
    pub secret: &'a str,
    pub email: &'a str,
    pub groups: &'a [i32],
}

#[derive(Identifiable, Queryable, Debug, Insertable, Associations,
    AsChangeset, Serialize, Deserialize)]
#[table_name="user_groups"]



pub struct UserGroup {
    pub id: i32,
    pub group_name: String,
    pub anonymous: bool,
}

#[derive(Identifiable, Queryable, Debug, Insertable, Associations,
    AsChangeset, Serialize, Deserialize)]
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

#[derive(Insertable, Queryable, Associations, AsChangeset, Identifiable, Debug, Serialize)]
#[table_name="skill_nuggets"]




pub struct SkillNugget {
    pub id: i32,
    pub skill_summary: String,
}

#[derive(Insertable)]
#[table_name="narrators"]
pub struct NewNarrator<'a> {
    pub name: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable,
    AsChangeset,Debug, Serialize, Deserialize)]
#[table_name="narrators"]

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
    pub file_sha2: &'a [u8],
}

#[derive(Insertable, Queryable, Associations,
Identifiable, Debug, Serialize, AsChangeset)]
#[table_name="audio_files"]
#[belongs_to(Narrator, foreign_key = "narrators_id")]
#[belongs_to(AudioBundle, foreign_key = "bundle_id")]

pub struct AudioFile {
    pub id: i32,
    pub narrators_id: i32,
    pub bundle_id: i32,
    pub file_path: String,
    pub mime: String,
    pub file_sha2: Option<Vec<u8>>,
}

#[derive(Queryable, AsChangeset, Debug, Serialize, Deserialize, Default)]
#[table_name="audio_files"]
pub struct UpdateAudioFile {
    #[serde(default)]
    pub narrators_id: Option<i32>,
    pub bundle_id: Option<i32>,
    pub file_path: Option<String>,
    pub mime: Option<String>,
    #[serde(deserialize_with = "double_option")]
    pub file_sha2: Option<Option<Vec<u8>>>,
}

#[derive(Insertable, Queryable, Associations, Identifiable,
    Debug, AsChangeset, Serialize, Deserialize)]
#[table_name="audio_bundles"]
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

#[derive(Insertable, Queryable, Associations, Identifiable, AsChangeset, Debug, Serialize)]
#[belongs_to(SkillNugget, foreign_key = "skill_id")]



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

#[derive(Queryable, AsChangeset, Debug, Serialize, Deserialize, Default)]
#[table_name="quiz_questions"]
#[serde(default)]
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

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Serialize, AsChangeset)]
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


#[derive(Queryable, AsChangeset, Debug, Serialize, Deserialize, Default)]
#[table_name="question_answers"]
#[serde(default)]
pub struct UpdateAnswer {
    pub question_id: Option<i32>,
    #[serde(deserialize_with = "double_option")]
    pub a_audio_bundle: Option<Option<i32>>,
    pub q_audio_bundle: Option<i32>,
    pub answer_text: Option<String>,
}

#[derive(Insertable, Queryable, Associations, Identifiable,
    Debug, Serialize, Deserialize, AsChangeset)]
#[belongs_to(SkillNugget, foreign_key = "skill_id")]



#[table_name="exercises"]
pub struct Exercise {
    pub id: i32,
    pub skill_id: i32,
    pub published: bool,
    pub skill_level: i32,
}

#[derive(Queryable, Debug, AsChangeset, Serialize, Deserialize, Default)]
#[table_name="exercises"]
#[serde(default)]
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

#[derive(Insertable, Identifiable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[belongs_to(Exercise, foreign_key = "exercise_id")]
#[belongs_to(Word, foreign_key = "id")]
#[table_name="exercise_variants"]
pub struct ExerciseVariant {
    pub id: i32,
    pub exercise_id: i32,
}

#[derive(Queryable, Debug, AsChangeset, Serialize, Deserialize, Default)]
#[table_name="exercise_variants"]
#[serde(default)]
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
    pub skill_level: i32,
    pub priority: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Serialize, AsChangeset)]
#[table_name="words"]
#[belongs_to(SkillNugget, foreign_key = "skill_nugget")]
#[belongs_to(AudioBundle, foreign_key = "audio_bundle")]


pub struct Word {
    pub id: i32,
    pub word: String,
    pub explanation: String,
    pub audio_bundle: i32,
    pub skill_nugget: i32,
    pub published: bool,
    pub skill_level: i32,
    pub priority: i32,
}

#[derive(Queryable, AsChangeset, Debug, Serialize, Deserialize, Default)]
#[table_name="words"]
#[serde(default)]
pub struct UpdateWord {
    pub word: Option<String>,
    pub explanation: Option<String>,
    pub audio_bundle: Option<i32>,
    pub skill_nugget: Option<i32>,
    pub published: Option<bool>,
    pub skill_level: Option<i32>,
    pub priority: Option<i32>,
}

#[derive(Insertable, Identifiable, Queryable, Associations, Debug,
AsChangeset, Serialize)]
#[table_name="due_items"]
#[belongs_to(User, foreign_key = "user_id")]


pub struct DueItem {
    pub id: i32,
    pub user_id: i32,
    pub due_date: DateTime<Utc>,
    pub due_delay: i32,
    pub cooldown_delay: DateTime<Utc>,
    pub correct_streak_overall: i32,
    pub correct_streak_this_time: i32,
    pub item_type: String,
}

#[derive(Insertable)]
#[table_name="due_items"]
pub struct NewDueItem<'a> {
    pub user_id: i32,
    pub due_date: DateTime<Utc>,
    pub due_delay: i32,
    pub cooldown_delay: DateTime<Utc>,
    pub correct_streak_overall: i32,
    pub correct_streak_this_time: i32,
    pub item_type: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug,
AsChangeset, Serialize, Deserialize)]
#[table_name="pending_items"]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(AudioFile, foreign_key = "audio_file_id")]



pub struct PendingItem {
    pub id: i32,
    pub user_id: i32,
    pub audio_file_id: i32,
    pub asked_date: DateTime<Utc>,
    pub pending: bool,
    pub item_type: String,
    pub test_item: bool,
}

#[derive(Insertable, Associations, Debug, AsChangeset)]
#[table_name="pending_items"]
pub struct NewPendingItem<'a> {
    pub user_id: i32,
    pub audio_file_id: i32,
    pub item_type: &'a str,
    pub test_item: bool,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, AsChangeset)]
#[table_name="q_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(QuizQuestion, foreign_key = "question_id")]
#[belongs_to(Answer, foreign_key = "correct_qa_id")]

pub struct QAskedData {
    pub id: i32,
    pub question_id: i32,
    pub correct_qa_id: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, Serialize)]
#[table_name="q_answered_data"]
#[belongs_to(QAskedData, foreign_key = "id")]
pub struct QAnsweredData {
    pub id: i32,
    pub answered_qa_id: Option<i32>,
    pub answered_date: DateTime<Utc>,
    pub active_answer_time_ms: i32,
    pub full_answer_time_ms: i32,
    pub full_spent_time_ms: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Clone, AsChangeset)]
#[table_name="e_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(Exercise, foreign_key = "exercise_id")]
#[belongs_to(Word, foreign_key = "word_id")]

pub struct EAskedData {
    pub id: i32,
    pub exercise_id: i32,
    pub word_id: i32,
}

#[derive(Insertable, Queryable, Associations, Identifiable,
    Debug, Clone, AsChangeset, Serialize)]
#[table_name="e_answered_data"]
#[belongs_to(EAskedData, foreign_key = "id")]
pub struct EAnsweredData {
    pub id: i32,
    pub answered_date: DateTime<Utc>,
    pub active_answer_time_ms: i32,
    pub full_answer_time_ms: i32,
    pub audio_times: i32,
    pub answer_level: i32,
    pub full_spent_time_ms: i32,
    pub reflected_time_ms: i32,
}

#[derive(Identifiable, Insertable, Queryable, Associations, Debug, Clone,
AsChangeset, Serialize, Deserialize)]
#[table_name="w_asked_data"]
#[belongs_to(PendingItem, foreign_key = "id")]
#[belongs_to(Word, foreign_key = "word_id")]

pub struct WAskedData {
    pub id: i32,
    pub word_id: i32,
    pub show_accents: bool,
}

#[derive(Identifiable, Insertable, Queryable, Associations,
    Debug, Clone, AsChangeset, Serialize)]
#[table_name="w_answered_data"]
#[belongs_to(WAskedData, foreign_key = "id")]
pub struct WAnsweredData {
    pub id: i32,
    pub full_spent_time_ms: i32,
    pub audio_times: i32,
    pub checked_date: DateTime<Utc>,
    pub active_answer_time_ms: i32,
}

#[derive(Insertable, Queryable, Associations, Debug,
AsChangeset, Serialize, Deserialize)]
#[table_name="question_data"]
#[belongs_to(DueItem, foreign_key = "due")]
pub struct QuestionData {
    pub question_id: i32,
    pub due: i32,
}

#[derive(Insertable, Queryable, Associations, Debug,
AsChangeset, Serialize, Deserialize)]
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

#[derive(Identifiable, Insertable, Queryable, Associations,
Debug, AsChangeset, Serialize, Deserialize)]
#[table_name="skill_data"]
#[primary_key(user_id, skill_nugget)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(SkillNugget, foreign_key = "skill_nugget")]
pub struct SkillData {
    pub user_id: i32,
    pub skill_nugget: i32,
    pub skill_level: i32,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset, Identifiable, Serialize)]
#[belongs_to(User, foreign_key = "id")]
#[table_name="user_metrics"]
pub struct UserMetrics {
    pub id: i32,
    pub new_words_since_break: i32,
    pub new_words_today: i32,
    pub quizes_since_break: i32,
    pub quizes_today: i32,
    pub break_until: DateTime<Utc>,
    pub today: DateTime<Utc>,
    pub max_words_since_break: i32,
    pub max_words_today: i32,
    pub max_quizes_since_break: i32,
    pub max_quizes_today: i32,

    pub break_length: i32,
    pub delay_multiplier: i32,
    pub initial_delay: i32,
    pub streak_limit: i32,
    pub cooldown_delay: i32,
    pub streak_skill_bump_criteria: i32,
}

#[derive(Debug, AsChangeset, Identifiable, Deserialize, Default)]
#[table_name="user_metrics"]
#[serde(default)]
pub struct UpdateUserMetrics {
    pub id: i32,
    pub new_words_since_break: Option<i32>,
    pub new_words_today: Option<i32>,
    pub quizes_since_break: Option<i32>,
    pub quizes_today: Option<i32>,
    pub break_until: Option<DateTime<Utc>>,
    pub today: Option<DateTime<Utc>>,
    pub max_words_since_break: Option<i32>,
    pub max_words_today: Option<i32>,
    pub max_quizes_since_break: Option<i32>,
    pub max_quizes_today: Option<i32>,

    pub break_length: Option<i32>,
    pub delay_multiplier: Option<i32>,
    pub initial_delay: Option<i32>,
    pub streak_limit: Option<i32>,
    pub cooldown_delay: Option<i32>,
    pub streak_skill_bump_criteria: Option<i32>,
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

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset,
    Identifiable, Serialize, Deserialize)]
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
    pub last_nag_email: Option<DateTime<Utc>>,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug, Serialize, Deserialize)]

#[belongs_to(UserGroup, foreign_key = "required_group")]
#[table_name="events"]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub published: bool,
    pub required_group: Option<i32>,
    pub priority: i32,
}

#[derive(Queryable, Identifiable, Serialize, Debug, AsChangeset, Deserialize, Default)]
#[table_name="events"]
#[serde(default)]
pub struct UpdateEvent {
    pub id: i32,
    pub name: Option<String>,
    pub published: Option<bool>,
    #[serde(deserialize_with = "double_option")]
    pub required_group: Option<Option<i32>>,
    pub priority: Option<i32>,
}

#[derive(Insertable)]
#[table_name="events"]
pub struct NewEvent<'a> {
    pub name: &'a str,
}

#[derive(Insertable, Queryable, Associations, Debug, AsChangeset, Serialize)]
#[table_name="event_experiences"]
#[changeset_options(treat_none_as_null = "true")]
#[belongs_to(Event, foreign_key = "event_id")]
#[belongs_to(User, foreign_key = "user_id")]
pub struct EventExperience {
    pub user_id: i32,
    pub event_id: i32,
    pub event_init: DateTime<Utc>,
    pub event_finish: Option<DateTime<Utc>>,
}

#[derive(Insertable)]
#[table_name="event_experiences"]
pub struct NewEventExperience {
    pub user_id: i32,
    pub event_id: i32,
}

#[derive(Insertable, Queryable, Identifiable, Associations, Debug, AsChangeset, Serialize)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(Event, foreign_key = "event_id")]
#[changeset_options(treat_none_as_null = "true")]
#[table_name="event_userdata"]
pub struct EventUserdata {
    pub id: i32,
    pub user_id: i32,
    pub event_id: i32,
    pub created: DateTime<Utc>,
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
    pub added: DateTime<Utc>,
}
