use super::schema::*;

use chrono::{DateTime, UTC};

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub email: &'a str,
}

#[has_many(passwords, foreign_key = "id")] // actually, the relationship is one-to-1..0
#[has_many(sessions, foreign_key = "user_id")]
#[derive(Identifiable, Queryable, Debug, Associations)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub joined: DateTime<UTC>,
}


#[derive(Identifiable, Queryable, Debug, Insertable, Associations)]
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
    pub sess_id: &'a [u8],
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: Vec<u8>,
}


#[derive(Identifiable, Queryable, Debug, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
pub struct Session {
    pub id: i32,
    pub sess_id: Vec<u8>,
    pub user_id: i32,
    pub started: DateTime<UTC>,
    pub last_seen: DateTime<UTC>,
    pub last_ip: Vec<u8>,
}


#[derive(Queryable, Debug)]
pub struct PendingEmailConfirm {
    pub secret: String,
    pub email: String,
    pub added: DateTime<UTC>,
}


#[derive(Insertable)]
#[table_name="pending_email_confirms"]
pub struct NewPendingEmailConfirm<'a> {
    pub secret: &'a str,
    pub email: &'a str,
}


#[derive(Insertable)]
#[table_name="skill_nuggets"]
pub struct NewSkillNugget<'a> {
    pub skill_summary: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[table_name="skill_nuggets"]
#[has_many(quiz_questions, foreign_key = "skill_id")]
pub struct SkillNugget {
    pub id: i32,
    pub skill_summary: String,
}

#[derive(Insertable)]
#[table_name="quiz_questions"]
pub struct NewQuizQuestion<'a> {
    pub skill_id: Option<i32>,
    pub q_name: &'a str,
    pub q_explanation: &'a str,
    pub question_text: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[belongs_to(SkillNugget, foreign_key = "skill_id")]
#[has_many(question_answers, foreign_key = "question_id")]
#[table_name="quiz_questions"]
pub struct QuizQuestion {
    pub id: i32,
    pub skill_id: Option<i32>,
    pub q_name: String,
    pub q_explanation: String,
    pub question_text: String,
}

#[derive(Insertable)]
#[table_name="narrators"]
pub struct NewNarrator<'a> {
    pub name: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[table_name="narrators"]
#[has_many(audio_files, foreign_key = "narrators_id")]
pub struct Narrator {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name="audio_files"]
pub struct NewAudioFile<'a> {
    pub narrators_id: i32,
    pub file_path: &'a str,
    pub mime: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[table_name="audio_files"]
#[belongs_to(Narrator, foreign_key = "narrators_id")]
#[has_many(question_audio, foreign_key = "id")]
#[has_many(question_answers, foreign_key = "audio_files_id")]
pub struct AudioFile {
    pub id: i32,
    pub narrators_id: i32,
    pub file_path: String,
    pub mime: String,
}

#[derive(Insertable)]
#[table_name="question_answers"]
pub struct NewAnswer<'a> {
    pub question_id: i32,
    pub audio_files_id: Option<i32>,
    pub answer_text: &'a str,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[table_name="question_answers"]
#[belongs_to(QuizQuestion, foreign_key = "question_id")]
#[belongs_to(AudioFile, foreign_key = "answer_audio")]
#[has_many(question_audio, foreign_key = "question_answers_id")]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub audio_files_id: Option<i32>,
    pub answer_text: String,
}

#[derive(Insertable, Queryable, Associations, Identifiable, Debug)]
#[table_name="question_audio"]
#[belongs_to(Answer, foreign_key = "question_answers_id")]
#[belongs_to(AudioFile, foreign_key = "id")]
pub struct QuestionAudio {
    pub id: i32,
    pub question_answers_id: i32,
}
