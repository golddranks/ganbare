
use super::*;
use chrono::UTC;
use pencil::{abort, jsonify, Response, redirect};
use pencil::helpers::{send_file, send_from_directory};
use rustc_serialize;
use regex;


use ganbare::audio;
use ganbare::quiz;
use ganbare::models;
use ganbare::skill;
use ganbare::manage;
use ganbare::event;
use ganbare::user;

pub fn get_audio(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

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
    let (file_name, mime_type) = audio::get_file(&conn, audio_id)
        .map_err(|e| {
            match e.kind() {
                &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                e => internal_error(e),
            }
        })?;

    use pencil::{PencilError, HTTPError};

    let mut file_path = AUDIO_DIR.clone();
    file_path.push(&file_name);

    send_file(&file_path.to_str().expect("The path SHOULD be valid unicode!"), mime_type, false, req.headers().get())
        .refresh_cookie(&sess)
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Audio file not found? The audio file database/folder is borked? {:?}", file_path); internal_error(e) },
            _ => internal_error(e),
        })
}

pub fn quiz_audio(req: &mut Request) -> PencilResult {

    let (conn, user, sess) = auth_user(req, "")?;

    let asked_id = req.view_args.get("audio_name").expect("Pencil guarantees that Line ID should exist as an arg.");

    let asked_id = asked_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    let (file_name, mime_type) = audio::for_quiz(&conn, &user, asked_id)
        .map_err(|e| {
            match e.kind() {
                &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                e => internal_error(e),
            }
        })?;

    use pencil::{PencilError, HTTPError};

    let mut file_path = AUDIO_DIR.clone();
    file_path.push(&file_name);

    send_file(&file_path.to_str().expect("The saved file path SHOULD be valid unicode!"), mime_type, false, req.headers().get())
        .refresh_cookie(&sess)
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Audio file not found? The audio file database/folder is borked? {:?}", file_path); internal_error(e) },
            _ => internal_error(e),
        })
}

pub fn get_image(req: &mut Request) -> PencilResult {

    let (_, _, sess) = auth_user(req, "")?;

    let file_name = req.view_args.get("filename").expect("Pencil guarantees that filename should exist as an arg.");

    use pencil::{PencilError, HTTPError};

    send_from_directory(IMAGES_DIR.to_str().expect("The image dir path should be valid unicode!"), &file_name, false, req.headers().get())
        .refresh_cookie(&sess)
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => { error!("Image file not found! {}", file_name); e },
            _ => internal_error(e),
        })
}

pub fn quiz_to_json(quiz: quiz::Quiz) -> PencilResult {
    use ganbare::quiz::Quiz::*;
    match quiz {
        Q(q_json) => jsonify(&q_json),
        E(e_json) => jsonify(&e_json),
        W(w_json) => jsonify(&w_json),
        F(future) => jsonify(&future),
    }
}

pub fn new_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    let new_quiz = quiz::get_new_quiz(&conn, &user).err_500()?;

    match new_quiz {

        Some(quiz) => quiz_to_json(quiz),

        None => jsonify(&()),

    }.refresh_cookie(&sess)
}

pub fn next_quiz(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    fn parse_answer(req : &mut Request) -> Result<quiz::Answered> {
        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let answer_type = &parse!(form.get("type"));

        if answer_type == "word" {
            let id = str::parse::<i32>(&parse!(form.get("asked_id")))?;
            let audio_times = str::parse::<i32>(&parse!(form.get("times_audio_played")))?;
            let active_answer_time_ms = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let full_spent_time_ms = str::parse::<i32>(&parse!(form.get("full_spent_time")))?;
            Ok(quiz::Answered::W(
                models::WAnsweredData{id, audio_times, checked_date: UTC::now(), active_answer_time_ms, full_spent_time_ms}
            ))
        } else if answer_type == "exercise" {
            let id = str::parse::<i32>(&parse!(form.get("asked_id")))?;
            let audio_times = str::parse::<i32>(&parse!(form.get("times_audio_played")))?;
            let active_answer_time_ms = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let reflected_time_ms = str::parse::<i32>(&parse!(form.get("reflected_time")))?;
            let full_answer_time_ms = str::parse::<i32>(&parse!(form.get("full_answer_time")))?;
            let full_spent_time_ms = str::parse::<i32>(&parse!(form.get("full_spent_time")))?;
            let answer_level = str::parse::<i32>(&parse!(form.get("answer_level")))?;
            Ok(quiz::Answered::E(
                models::EAnsweredData{id, audio_times, active_answer_time_ms, answered_date: UTC::now(), reflected_time_ms, full_answer_time_ms, answer_level, full_spent_time_ms}
            ))
        } else if answer_type == "question" {
            let id = str::parse::<i32>(&parse!(form.get("asked_id")))?;
            let answered_qa_id = str::parse::<i32>(&parse!(form.get("answered_qa_id")))?;
            let answered_qa_id = if answered_qa_id > 0 { Some(answered_qa_id) } else { None }; // Negatives mean that question was unanswered (due to time limit)
            let active_answer_time_ms = str::parse::<i32>(&parse!(form.get("active_answer_time")))?;
            let full_answer_time_ms = str::parse::<i32>(&parse!(form.get("full_answer_time")))?;
            let full_spent_time_ms = str::parse::<i32>(&parse!(form.get("full_spent_time")))?;
            Ok(quiz::Answered::Q(
                models::QAnsweredData{id, answered_qa_id, answered_date: UTC::now(), active_answer_time_ms, full_answer_time_ms, full_spent_time_ms}      
            ))
        } else {
            Err(ErrorKind::FormParseError.into())
        }
    };

    let answer = err_400!(parse_answer(req), "Can't parse form data? {:?}", req.form());

    let new_quiz = quiz::get_next_quiz(&conn, &user, answer).err_500()?;

    match new_quiz {

        Some(quiz) => quiz_to_json(quiz),

        None => jsonify(&()),

    }.refresh_cookie(&sess)
}


pub fn get_item(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_word" => {
            let item = manage::get_word(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
                },
        "get_question" => {
            let item = manage::get_question(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        },
        "get_exercise" => {
            let item = manage::get_exercise(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        },
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn del_item(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    info!("del_item");

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "del_narrator" => {
            if !audio::del_narrator(&conn, id).err_500()? {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_bundle" => {
            if !audio::del_bundle(&conn, id).err_500()? {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_user" => {
            if user::deactivate_user(&conn, id).err_500()?.is_none() {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_skill" => {
            if skill::remove(&conn, id).err_500()?.is_none() {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_word" => {
            if manage::remove_word(&conn, id).err_500()?.is_none() {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_question" => {
            if !manage::remove_question(&conn, id).err_500()? {
                 return abort(404);
            }
            jsonify(&())
        },
        "del_exercise" => {
            if !manage::remove_exercise(&conn, id).err_500()?{
                 return abort(404);
            }
            jsonify(&())
        },
        _ => return Err(internal_error("no such endpoint!")),
    };


    json.refresh_cookie(&sess)
}

pub fn merge_item(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    info!("merge_item");

    let id_from = req.view_args.get("id_from").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id_from = id_from.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let id_to = req.view_args.get("id_to").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id_to = id_to.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "merge_narrator" => {
            audio::merge_narrator(&conn, id_from, id_to).err_500()?;
            jsonify(&())
        },
        "merge_bundle" => {
            audio::merge_audio_bundle(&conn, id_from, id_to).err_500()?;
            jsonify(&())
        },
        _ => return Err(internal_error("no such endpoint!")),
    };


    json.refresh_cookie(&sess)
}

pub fn get_all(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_nuggets" => {
            let items = skill::get_skill_nuggets(&conn).err_500()?;
            jsonify(&items)
        },
        "get_bundles" => {
            let items = audio::get_all_bundles(&conn).err_500()?;
            jsonify(&items)
        },
        "get_users" => {
            let items = ganbare::user::get_all(&conn).err_500()?;
            jsonify(&items)
        },
        "get_narrators" => {
            let items = audio::get_narrators(&conn).err_500()?;
            jsonify(&items)
        },
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn set_published(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "editors")?;

    let id = req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    match endpoint.as_ref() {
        "publish_words" => {
            manage::publish_word(&conn, id, true).err_500()?;
        },
        "publish_questions" => {
            manage::publish_question(&conn, id, true).err_500()?;
        },
        "publish_exercises" => {
            manage::publish_exercise(&conn, id, true).err_500()?;
        },
        "unpublish_words" => {
            manage::publish_word(&conn, id, false).err_500()?;
        },
        "unpublish_questions" => {
            manage::publish_question(&conn, id, false).err_500()?;
        },
        "unpublish_exercises" => {
            manage::publish_exercise(&conn, id, false).err_500()?;
        },
        _ => return Err(internal_error("no such endpoint!")),
    };
    let mut resp = Response::new_empty();
    resp.status_code = 204;
    resp.refresh_cookie(&sess)
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
        
            let updated_item = try_or!(manage::update_word(&conn, id, item, &*IMAGES_DIR).err_500()?, else return abort(404));

            json = jsonify(&updated_item);

        },
        "update_question" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(manage::update_question(&conn, id, item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_answer" => {

            let item = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
        
            let updated_item = try_or!(manage::update_answer(&conn, id, item, &*IMAGES_DIR).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_bundle" => {

            let item: ganbare::models::AudioBundle = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
            if item.id != id {
                return abort(400);
            }
            let updated_item = try_or!(audio::change_bundle_name(&conn, id, &item.listname).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        "update_narrator" => {

            let item: ganbare::models::Narrator = rustc_serialize::json::decode(&text)
                            .map_err(|_| abort(400).unwrap_err())?;
            if item.id != id {
                return abort(400);
            }
            let updated_item = try_or!(audio::change_narrator_name(&conn, id, &item.name).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        },
        _ => return Err(internal_error("no such endpoint!")),
    }
    
    json.refresh_cookie(&sess)
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

    let id = manage::post_question(&conn, new_qq, new_aas).err_500()?;
        
    let new_url = format!("/api/questions/{}", id);

    redirect(&new_url, 303).refresh_cookie(&sess)
}

pub fn post_exercise(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "editors")?;

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    use ganbare::models::{UpdateExercise, UpdateExerciseVariant, NewExercise, ExerciseVariant};

    let (qq, aas) : (UpdateExercise, Vec<UpdateExerciseVariant>) = err_400!(rustc_serialize::json::decode(&text),
            "Error when parsing the JSON.");

    fn parse_qq(qq: &UpdateExercise) -> Result<NewExercise> {
        let qq = NewExercise {
            skill_id: qq.skill_id.ok_or(ErrorKind::FormParseError.to_err())?,
            skill_level: qq.skill_level.ok_or(ErrorKind::FormParseError.to_err())?,
        };
        Ok(qq)
    }

    fn parse_aa(aa: &UpdateExerciseVariant) -> Result<ExerciseVariant> {
        let aa = ExerciseVariant {
            id: aa.id.ok_or(ErrorKind::FormParseError.to_err())?,
            exercise_id: aa.exercise_id.ok_or(ErrorKind::FormParseError.to_err())?,
        };
        Ok(aa)
    }

    let new_qq = err_400!(parse_qq(&qq), "Fields missing from UpdateExercise");

    let mut new_aas = vec![];
    for aa in &aas {
        let new_aa = err_400!(parse_aa(aa), "Fields missing from UpdateExerciseVariant");
        new_aas.push(new_aa);
    }

    let id = manage::post_exercise(&conn, new_qq, new_aas).err_500()?;
        
    let new_url = format!("/api/exercises/{}", id);

    redirect(&new_url, 303).refresh_cookie(&sess)
}

pub fn save_eventdata(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    let eventname = req.view_args.remove("eventname").expect("Pencil guarantees that Line ID should exist as an arg.");
    let key = req.view_args.remove("key");
    let (event, _) = event::require_ongoing(&conn, &eventname, &user).err_401()?;

    use std::io::Read;
    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    event::save_userdata(&conn, &event, &user, key.as_ref().map(|s|&**s), &text).err_500()?;

    Response::from("OK.").refresh_cookie(&sess)
}

pub fn user(req: &mut Request) -> PencilResult {
    let (conn, _, sess) = auth_user(req, "admins")?;

    let user_id = req.view_args.remove("user_id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let user_id = user_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "add_group" => {

            let group_id = req.view_args.remove("group_id").expect("Pencil guarantees that Line ID should exist as an arg.");
            let group_id = group_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

            ganbare::user::join_user_group_by_id(&conn, user_id, group_id).err_500()?;
            jsonify(&())
                },
        "remove_group" => {

            let group_id = req.view_args.remove("group_id").expect("Pencil guarantees that Line ID should exist as an arg.");
            let group_id = group_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

            ganbare::user::remove_user_group_by_id(&conn, user_id, group_id).err_500()?;
            jsonify(&())
                },
        "set_metrics" => {

            use std::io::Read;
            use ganbare::models::UpdateUserMetrics;

            let mut text = String::new();
            req.read_to_string(&mut text).err_500()?;
            let metrics: UpdateUserMetrics = err_400!(rustc_serialize::json::decode(&text), "Can't decode JSON: {:?}", &text);

            if user_id != metrics.id {
                return Ok(bad_request("user id in the URL must be the same as in the JSON content!"));
            }
        
            ganbare::user::set_metrics(&conn, &metrics).err_500()?;
            jsonify(&())
                },
        _ => {
            return Err(internal_error("no such endpoint!"))
        },
    };

    json.refresh_cookie(&sess)
}

