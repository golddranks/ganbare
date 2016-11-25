
use super::*;
use pencil::{abort, jsonify, Response, redirect};
use rand;
use pencil::helpers::{send_file, send_from_directory};
use rustc_serialize;
use regex;
use unicode_normalization::UnicodeNormalization;

pub fn get_audio(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let mut audio_name = req.view_args.get("audio_name").expect("Pencil guarantees that Line ID should exist as an arg.").split('.');
    let audio_id = try_or!(audio_name.next(), else return abort(404));
    let audio_extension = try_or!(audio_name.next(), else return abort(404));
    if audio_extension != "mp3" {
        return abort(404);
    }
    if audio_name.next().is_some() {
        return abort(404);
    }
    let audio_id = audio_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let (file_name, mime_type) = ganbare::get_audio_file(&conn, audio_id)
        .map_err(|e| {
            match e.kind() {
                &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                _ => abort(500).unwrap_err(),
            }
        })?;

    use pencil::{PencilError, HTTPError};

    let file_path = AUDIO_DIR.to_string() + "/" + &file_name;

    send_file(&file_path, mime_type, false)
        .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Audio file not found? The audio file database/folder is borked? {}", file_path); internal_error(e) },
            _ => { internal_error(e) }
        })

  //  return abort(500);
}

pub fn get_image(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let file_name = req.view_args.get("filename").expect("Pencil guarantees that filename should exist as an arg.");

    use pencil::{PencilError, HTTPError};

    send_from_directory(&*IMAGES_DIR, &file_name, false)
        .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Image file not found! {}", file_name); e },
            _ => { internal_error(e) }
        })
}

#[derive(RustcEncodable)]
struct QuestionJson {
    quiz_type: String,
    question_id: i32,
    explanation: String,
    question: (String, i32),
    right_a: i32,
    answers: Vec<(i32, String, Option<i32>)>,
    due_delay: i32,
    due_date: Option<String>,
}

#[derive(RustcEncodable)]
struct WordJson {
    quiz_type: String,
    show_accents: bool,
    id: i32,
    word: String,
    explanation: String,
    audio_id: i32,
}

#[derive(RustcEncodable)]
struct ExerciseJson {
    quiz_type: String,
    id: i32,
    word: String,
    explanation: String,
    audio_id: i32,
    due_delay: i32,
    due_date: Option<String>,
}

pub fn quiz_to_json(quiz: ganbare::Quiz) -> PencilResult {
    use rand::Rng;
    use ganbare::Quiz::*;
    let mut rng = rand::thread_rng();
    match quiz {
    Question(ganbare::Question{ question, question_audio, right_answer_id, answers, due_delay, due_date }) => {

        let mut answers_json = Vec::with_capacity(answers.len());

        let chosen_q_audio = rng.choose(&question_audio).expect("Audio for a Question: Shouldn't be empty! Borked database?");
        

        for a in answers {
            answers_json.push((a.id, a.answer_text, a.a_audio_bundle));
        }

        rng.shuffle(&mut answers_json);
        

        jsonify(&QuestionJson {
            quiz_type: "question".into(),
            question_id: question.id,
            explanation: question.q_explanation,
            question: (question.question_text, chosen_q_audio.id),
            right_a: right_answer_id,
            answers: answers_json,
            due_delay,
            due_date: due_date.map(|d| d.to_rfc3339()),
        })
    },
    Exercise(ganbare::Exercise { word: ganbare::models::Word { id, word, explanation, .. }, due_date, due_delay, audio_files }) => {

        let chosen_audio = rng.choose(&audio_files).expect("Audio for a Exercise: Shouldn't be empty! Borked database?");

        jsonify(&ExerciseJson {
            quiz_type: "exercise".into(),
            id,
            word: word.nfc().collect::<String>(), // Unicode normalization, because "word" is going to be accented kana-by-kana.
            explanation,
            audio_id: chosen_audio.id,
            due_delay,
            due_date: due_date.map(|d| d.to_rfc3339()),
        })
    },
    Word((ganbare::models::Word { id, word, explanation, .. }, audio_files, show_accents)) => {

        let chosen_audio = rng.choose(&audio_files).expect("Audio for a Word: Shouldn't be empty! Borked database?");

        jsonify(&WordJson {
            show_accents,
            quiz_type: "word".into(),
            id,
            word: word.nfc().collect::<String>(), // Unicode normalization, because "word" is going to be accented kana-by-kana.
            explanation,
            audio_id: chosen_audio.id,
        })
    },
    }
}

pub fn new_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    let new_quiz = ganbare::get_new_quiz(&conn, &user).err_500()?;

    let quiz = try_or!{new_quiz, else return jsonify(&())}; 

    quiz_to_json(quiz).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn next_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    fn parse_answer(req : &mut Request) -> Result<ganbare::Answered> {
        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let answer_type = &parse!(form.get("type"));

        if answer_type == "word" {
            let word_id = str::parse::<i32>(&parse!(form.get("word_id")))?;
            let times_audio_played = str::parse::<i32>(&parse!(form.get("times_audio_played")))?;
            let time = str::parse::<i32>(&parse!(form.get("time")))?;
            Ok(ganbare::Answered::Word(
                ganbare::AnsweredWord{word_id, times_audio_played, time}
            ))
        } else if answer_type == "exercise" {
            let word_id = str::parse::<i32>(&parse!(form.get("word_id")))?;
            let times_audio_played = str::parse::<i32>(&parse!(form.get("times_audio_played")))?;
            let active_answer_time = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let full_answer_time = str::parse::<i32>(&parse!(form.get("full_answer_time")))?;
            let correct = str::parse::<bool>(&parse!(form.get("correct")))?;
            Ok(ganbare::Answered::Exercise(
                ganbare::AnsweredExercise{word_id, times_audio_played, active_answer_time, full_answer_time, correct}
            ))
        } else if answer_type == "question" {
            let question_id = str::parse::<i32>(&parse!(form.get("question_id")))?;
            let right_answer_id = str::parse::<i32>(&parse!(form.get("right_a_id")))?;
            let answered_id = str::parse::<i32>(&parse!(form.get("answered_id")))?;
            let answered_id = if answered_id > 0 { Some(answered_id) } else { None }; // Negatives mean that question was unanswered (due to time limit)
            let q_audio_id = str::parse::<i32>(&parse!(form.get("q_audio_id")))?;
            let active_answer_time = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let full_answer_time = str::parse::<i32>(&parse!(form.get("full_answer_time")))?;
            Ok(ganbare::Answered::Question(
                ganbare::AnsweredQuestion{question_id, right_answer_id, answered_id, q_audio_id, active_answer_time, full_answer_time}
            ))
        } else {
            Err(ErrorKind::FormParseError.into())
        }
    };

    let answer = parse_answer(req)
        .map_err(|_| abort(400).unwrap_err())?;

    let new_quiz = ganbare::get_next_quiz(&conn, &user, answer)
        .err_500()?;

    let quiz = try_or!{new_quiz, else return jsonify(&())}; 
    quiz_to_json(quiz).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn get_item(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_word" => {
            let item = ganbare::get_word(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
                },
        "get_question" => {
            let item = ganbare::get_question(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn get_all(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_nuggets" => {
            let items = ganbare::get_skill_nuggets(&conn).err_500()?;
            jsonify(&items)
        },
        "get_bundles" => {
            let items = ganbare::get_audio_bundles(&conn).err_500()?;
            jsonify(&items)
        },
        _ => {
            return abort(500)
        },
    };

    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn set_published(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    match endpoint.as_ref() {
        "publish_words" => {
            ganbare::publish_word(&conn, id, true).err_500()?;
        },
        "publish_questions" => {
            ganbare::publish_question(&conn, id, true).err_500()?;
        },
        "unpublish_words" => {
            ganbare::publish_word(&conn, id, false).err_500()?;
        },
        "unpublish_questions" => {
            ganbare::publish_question(&conn, id, false).err_500()?;
        },
        _ => {
            return abort(500)
        },
    };
    let mut resp = Response::new_empty();
    resp.status_code = 204;
    Ok(resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}

pub fn update_item(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.")
                .parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    lazy_static! {
        // Taking JSON encoding into account: " is escaped as \"
        static ref RE: regex::Regex = regex::Regex::new(r###"<img ([^>]* )?src=\\"(?P<src>[^"]*)\\"( [^>]*)?>"###).unwrap();
    }
    let text = RE.replace_all(&text, r###"<img src=\"$src\">"###);

    let json;
    match endpoint.as_str() {
        "update_word" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_word(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);

        },
        "update_question" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_question(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_answer" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(ganbare::update_answer(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        _ => return abort(500),
    }
    
    json.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
}


pub fn post_question(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    use ganbare::models::{UpdateQuestion, UpdateAnswer, NewQuizQuestion, NewAnswer};

    let (qq, aas) : (UpdateQuestion, Vec<UpdateAnswer>) = rustc_serialize::json::decode(&text)
            .map_err(|_| abort(400).unwrap_err())?;

    fn parse_qq(qq: &UpdateQuestion) -> Result<NewQuizQuestion> {
        let qq = NewQuizQuestion {
            skill_id: qq.skill_id.ok_or(ErrorKind::FormParseError.to_err())?,
            q_name: qq.q_name.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            q_explanation: qq.q_explanation.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            question_text: qq.question_text.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
            skill_level: qq.skill_level.ok_or(ErrorKind::FormParseError.to_err())?,
        };
        Ok(qq)
    }

    fn parse_aa(aa: &UpdateAnswer) -> Result<NewAnswer> {
        let aa = NewAnswer {
            question_id: aa.question_id.ok_or(ErrorKind::FormParseError.to_err())?,
            a_audio_bundle: aa.a_audio_bundle.unwrap_or(None),
            q_audio_bundle: aa.q_audio_bundle.ok_or(ErrorKind::FormParseError.to_err())?,
            answer_text: aa.answer_text.as_ref().ok_or(ErrorKind::FormParseError.to_err())?.as_str(),
        };
        Ok(aa)
    }

    let new_qq = parse_qq(&qq)
            .map_err(|_| abort(400).unwrap_err())?;

    let mut new_aas = vec![];
    for aa in &aas {
        let new_aa = parse_aa(aa)
            .map_err(|_| abort(400).unwrap_err())?;
        new_aas.push(new_aa);
    }

    let id = ganbare::post_question(&conn, new_qq, new_aas).err_500()?;
        
    let new_url = format!("/api/questions/{}", id);

    redirect(&new_url, 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

