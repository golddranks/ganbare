
use super::*;
use chrono::offset::Utc;
use diesel::Connection;
use pencil::{abort, jsonify, Response, redirect};
use pencil::helpers::{send_file_range, send_from_directory_range};
use regex;
use std::io::{self, Read};
use hyper::header::ContentLength;
use serde_json;
use ganbare_backend::time_it;
use log::{info, debug};
use lazy_static::lazy_static;

use crate::{err_400, try_or, parse};
use ganbare::audio;
use ganbare::quiz;
use ganbare::models;
use ganbare::skill;
use ganbare::manage;
use ganbare::event;
use ganbare::user;
use test;

pub fn get_audio(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "editors")?;

    let mut audio_name = req.view_args
        .get("audio_name")
        .expect("Pencil guarantees that Line ID should exist as an arg.")
        .split('.');
    let audio_id = try_or!(audio_name.next(), else return abort(404));
    let audio_extension = try_or!(audio_name.next(), else return abort(404));
    if audio_extension != "mp3" {
        return abort(404);
    }
    if audio_name.next().is_some() {
        return abort(404);
    }
    let audio_id =
        audio_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let (file_name, mime_type) = audio::get_file_path(&conn, audio_id).map_err(|e| match e.kind() {
                     &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                     e => internal_error(e),
                 })?;

    use pencil::{PencilError, HTTPError};

    let mut file_path = AUDIO_DIR.clone();
    file_path.push(&file_name);

    time_it!("get_audio",
             send_file_range(file_path.to_str().expect("The path SHOULD be valid unicode!"),
                             mime_type,
                             false,
                             req.headers().get()))
            .set_static_cache()
            .refresh_cookie(&sess)
            .map_err(|e| match e {
                         PencilError::PenHTTPError(HTTPError::NotFound) => {
                error!("Audio file not found? The audio file database/folder is borked? {:?}",
                       file_path);
                internal_error(e)
            }
                         _ => internal_error(e),
                     })
}

pub fn get_build_number(_: &mut Request) -> PencilResult {
    jsonify(&get_version_info())
}

pub fn quiz_audio(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "")?;

    let asked_id = req.view_args
        .get("audio_name")
        .expect("Pencil guarantees that Line ID should exist as an arg.");

    let asked_id =
        asked_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    let (file_name, mime_type) = audio::for_quiz(&conn, sess.user_id, asked_id).map_err(|e| match e.kind() {
                     &ErrorKind::FileNotFound => abort(404).unwrap_err(),
                     e => internal_error(e),
                 })?;

    use pencil::{PencilError, HTTPError};

    let mut file_path = AUDIO_DIR.clone();
    file_path.push(&file_name);

    time_it!("quiz_audio",
             send_file_range(file_path.to_str().expect("The saved file path SHOULD be valid unicode!"),
                       mime_type,
                       false,
                       req.headers().get()))
        .set_static_cache()
        .refresh_cookie(&sess)
        .map_err(|e| match e {
            PencilError::PenHTTPError(HTTPError::NotFound) => {
                error!("Audio file not found? The audio file database/folder is borked? {:?}",
                       file_path);
                internal_error(e)
            }
            _ => internal_error(e),
        })
}

pub fn get_image(req: &mut Request) -> PencilResult {

    let (_, sess) = auth_user(req, "")?;

    let file_name = req.view_args
        .get("filename")
        .expect("Pencil guarantees that filename should exist as an arg.");

    use pencil::{PencilError, HTTPError};

    time_it!("get_image",
             send_from_directory_range(IMAGES_DIR.to_str()
                                     .expect("The image dir path should be valid unicode!"),
                                 file_name,
                                 false,
                                 req.headers().get()))
            .set_static_cache()
            .refresh_cookie(&sess)
            .map_err(|e| match e {
                         PencilError::PenHTTPError(HTTPError::NotFound) => {
                error!("Image file not found! {}", file_name);
                e
            }
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
    let (conn, sess) = auth_user(req, "")?;

    let new_quiz = time_it!("new_quiz",
                            quiz::get_new_quiz(&conn, sess.user_id).err_500())?;

    match new_quiz {
            Some(quiz) => quiz_to_json(quiz),
            None => jsonify(&()),
        }
        .refresh_cookie(&sess)
}

pub fn new_quiz_testing(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;

    let new_quiz = if let Some((ev, _)) = time_it!("is_ongoing pretest",
                                                   ganbare::event::is_ongoing(&conn,
                                                                              "pretest",
                                                                              sess.user_id)
                                                           .err_500())? {
        debug!("Pretest questions!");
        test::get_new_quiz_pretest(&conn, sess.user_id, &ev).err_500()?
    } else if let Some((ev, _)) =
        time_it!("is_ongoing posttest",
                 ganbare::event::is_ongoing(&conn, "posttest", sess.user_id).err_500())? {
        debug!("Posttest questions!");
        test::get_new_quiz_posttest(&conn, sess.user_id, &ev).err_500()?
    } else if let Some((ev, _)) = ganbare::event::is_ongoing(&conn, "simple_test", sess.user_id).err_500()? {
        debug!("Simple test questions!");
        test::get_new_quiz_pretest(&conn, sess.user_id, &ev).err_500()?
    } else {
        None
    };

    match new_quiz {
            Some(quiz) => quiz_to_json(quiz),
            None => jsonify(&()),
        }
        .refresh_cookie(&sess)
}

fn parse_next_quiz_answer(req: &mut Request) -> Result<quiz::Answered> {
    let form = req.form();
    let answer_type = parse!(form.get::<str>("type"));

    if answer_type == "word" {
        let id = str::parse::<i32>(parse!(form.get("asked_id")))?;
        let audio_times = str::parse::<i32>(parse!(form.get("times_audio_played")))?;
        let active_answer_time_ms = str::parse::<i32>(parse!(form.get("active_answer_time")))?;
        let full_spent_time_ms = str::parse::<i32>(parse!(form.get("full_spent_time")))?;
        Ok(quiz::Answered::W(models::WAnsweredData {
                                 id: id,
                                 audio_times: audio_times,
                                 checked_date: Utc::now(),
                                 active_answer_time_ms: active_answer_time_ms,
                                 full_spent_time_ms: full_spent_time_ms,
                             }))
    } else if answer_type == "exercise" {
        let id = str::parse::<i32>(parse!(form.get("asked_id")))?;
        let audio_times = str::parse::<i32>(parse!(form.get("times_audio_played")))?;
        let active_answer_time_ms = str::parse::<i32>(parse!(form.get("active_answer_time")))?;
        let reflected_time_ms = str::parse::<i32>(parse!(form.get("reflected_time")))?;
        let full_answer_time_ms = str::parse::<i32>(parse!(form.get("full_answer_time")))?;
        let full_spent_time_ms = str::parse::<i32>(parse!(form.get("full_spent_time")))?;
        let answer_level = str::parse::<i32>(parse!(form.get("answer_level")))?;
        Ok(quiz::Answered::E(models::EAnsweredData {
                                 id: id,
                                 audio_times: audio_times,
                                 active_answer_time_ms: active_answer_time_ms,
                                 answered_date: Utc::now(),
                                 reflected_time_ms: reflected_time_ms,
                                 full_answer_time_ms: full_answer_time_ms,
                                 answer_level: answer_level,
                                 full_spent_time_ms: full_spent_time_ms,
                             }))
    } else if answer_type == "question" {
        let id = str::parse::<i32>(parse!(form.get("asked_id")))?;
        let answered_qa_id = str::parse::<i32>(parse!(form.get("answered_qa_id")))?;
        let answered_qa_id = if answered_qa_id > 0 {
            Some(answered_qa_id)
        } else {
            None
        }; // Negatives mean that question was unanswered (due to time limit)
        let active_answer_time_ms = str::parse::<i32>(parse!(form.get("active_answer_time")))?;
        let full_answer_time_ms = str::parse::<i32>(parse!(form.get("full_answer_time")))?;
        let full_spent_time_ms = str::parse::<i32>(parse!(form.get("full_spent_time")))?;
        Ok(quiz::Answered::Q(models::QAnsweredData {
                                 id: id,
                                 answered_qa_id: answered_qa_id,
                                 answered_date: Utc::now(),
                                 active_answer_time_ms: active_answer_time_ms,
                                 full_answer_time_ms: full_answer_time_ms,
                                 full_spent_time_ms: full_spent_time_ms,
                             }))
    } else {
        Err(ErrorKind::FormParseError.into())
    }
}

pub fn next_quiz(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;

    let answer = err_400!(parse_next_quiz_answer(req),
                          "Can't parse form data? {:?}",
                          req.form());
    
    let new_quiz = time_it!("next_quiz",
                            conn.transaction(||
                                quiz::get_next_quiz(&conn, sess.user_id, answer)
                            ).err_500_debug(sess.user_id, &*req)
                        )?;

    match new_quiz {
            Some(quiz) => quiz_to_json(quiz),
            None => jsonify(&()),
        }
        .refresh_cookie(&sess)
}

pub fn next_quiz_testing(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;
    let answer = err_400!(parse_next_quiz_answer(req),
                          "Can't parse form data? {:?}",
                          req.form());

    let new_quiz = if let Some((ev, _)) =
        ganbare::event::is_ongoing(&conn, "pretest", sess.user_id).err_500()? {
        test::get_next_quiz_pretest(&conn, sess.user_id, answer, &ev).err_500()?
    } else if let Some((ev, _)) =
        ganbare::event::is_ongoing(&conn, "posttest", sess.user_id).err_500()? {
        test::get_next_quiz_posttest(&conn, sess.user_id, answer, &ev).err_500()?
    } else if let Some((ev, _)) =
        ganbare::event::is_ongoing(&conn, "simple_test", sess.user_id).err_500()? {
        test::get_next_quiz_pretest(&conn, sess.user_id, answer, &ev).err_500()?
    } else {
        None
    };

    match new_quiz {
            Some(quiz) => quiz_to_json(quiz),
            None => jsonify(&()),
        }
        .refresh_cookie(&sess)
}


pub fn get_item(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    let id =
        req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_word" => {
            let item = manage::get_word(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        }
        "get_question" => {
            let item = manage::get_question(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        }
        "get_exercise" => {
            let item = manage::get_exercise(&conn, id).err_500()?
                .ok_or_else(|| abort(404).unwrap_err())?;
            jsonify(&item)
        }
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn del_item(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    info!("del_item");

    let id =
        req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "del_narrator" => {
            if !audio::del_narrator(&conn, id).err_500()? {
                return abort(404);
            }
            jsonify(&())
        }
        "del_bundle" => {
            if !audio::del_bundle(&conn, id).err_500()? {
                return abort(404);
            }
            jsonify(&())
        }
        "del_user" => {
            if user::deactivate_user(&conn, id).err_500()?.is_none() {
                return abort(404);
            }
            jsonify(&())
        }
        "del_due_and_pending_items" => {
            manage::del_due_and_pending_items(&conn, id).err_500()?;
            jsonify(&())
        }
        "del_skill" => {
            if skill::remove(&conn, id).err_500()?.is_none() {
                return abort(404);
            }
            jsonify(&())
        }
        "del_word" => {
            if manage::remove_word(&conn, id).err_500()?.is_none() {
                return abort(404);
            }
            jsonify(&())
        }
        "del_question" => {
            if !manage::remove_question(&conn, id).err_500()? {
                return abort(404);
            }
            jsonify(&())
        }
        "del_exercise" => {
            if !manage::remove_exercise(&conn, id).err_500()? {
                return abort(404);
            }
            jsonify(&())
        }
        "del_event_exp" => {
            let user_id = req.view_args
                .get("user_id")
                .expect("Pencil guarantees that Line ID should exist as an arg.");
            let user_id = user_id.parse::<i32>()
                .expect("Pencil guarantees that Line ID should be an integer.");
            if !event::remove_exp(&conn, id, user_id).err_500()? {
                return abort(404);
            }
            jsonify(&())
        }
        _ => return Err(internal_error("no such endpoint!")),
    };


    json.refresh_cookie(&sess)
}

pub fn merge_item(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    info!("merge_item");

    let id_from = req.view_args
        .get("id_from")
        .expect("Pencil guarantees that Line ID should exist as an arg.");
    let id_from =
        id_from.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let id_to =
        req.view_args.get("id_to").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id_to = id_to.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "merge_narrator" => {
            audio::merge_narrator(&conn, id_from, id_to).err_500()?;
            jsonify(&())
        }
        "merge_bundle" => {
            audio::merge_audio_bundle(&conn, id_from, id_to).err_500()?;
            jsonify(&())
        }
        _ => return Err(internal_error("no such endpoint!")),
    };


    json.refresh_cookie(&sess)
}

pub fn get_all(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_nuggets" => {
            let items = skill::get_skill_nuggets(&conn).err_500()?;
            jsonify(&items)
        }
        "get_bundles" => {
            let items = audio::get_all_bundles(&conn).err_500()?;
            jsonify(&items)
        }
        "get_users" => {
            let items = ganbare::user::get_all(&conn).err_500()?;
            jsonify(&items)
        }
        "get_narrators" => {
            let items = audio::get_narrators(&conn).err_500()?;
            jsonify(&items)
        }
        "get_groups" => {
            let items = ganbare::user::all_groups(&conn).err_500()?;
            jsonify(&items)
        }
        "get_events" => {
            let items = ganbare::event::get_all(&conn).err_500()?;
            jsonify(&items)
        }
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn get_user_details(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    let id =
        req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "get_skills" => {
            let items = skill::get_skill_data(&conn, id).err_500()?;
            jsonify(&items)
        }
        "get_asked_items" => {
            let items = skill::get_asked_items(&conn, id).err_500()?;
            jsonify(&items)
        }
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn set_published(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "editors")?;

    let id =
        req.view_args.get("id").expect("Pencil guarantees that Line ID should exist as an arg.");
    let id = id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    match endpoint.as_ref() {
        "publish_words" => {
            manage::publish_word(&conn, id, true).err_500()?;
        }
        "publish_questions" => {
            manage::publish_question(&conn, id, true).err_500()?;
        }
        "publish_exercises" => {
            manage::publish_exercise(&conn, id, true).err_500()?;
        }
        "unpublish_words" => {
            manage::publish_word(&conn, id, false).err_500()?;
        }
        "unpublish_questions" => {
            manage::publish_question(&conn, id, false).err_500()?;
        }
        "unpublish_exercises" => {
            manage::publish_exercise(&conn, id, false).err_500()?;
        }
        _ => return Err(internal_error("no such endpoint!")),
    };
    let mut resp = Response::new_empty();
    resp.status_code = 204;
    resp.refresh_cookie(&sess)
}

pub fn update_item(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "editors")?;

    let id = req.view_args
        .get("id")
        .expect("Pencil guarantees that Line ID should exist as an arg.")
        .parse::<i32>()
        .expect("Pencil guarantees that Line ID should be an integer.");

    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    lazy_static! {
        // Taking JSON encoding into account: " is escaped as \"
        static ref RE: regex::Regex =
            regex::Regex::new(r##"<img ([^>]* )?src=\\"(?P<src>[^"]*)\\"( [^>]*)?>"##).unwrap();
    }
    let text = RE.replace_all(&text, r###"<img src=\"$src\">"###);

    let json;
    match endpoint.as_str() {
        "update_word" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                manage::update_word(&conn, id, item, &*IMAGES_DIR).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);

        }
        "update_exercise" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                manage::update_exercise(&conn, id, item).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_question" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                manage::update_question(&conn, id, item).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_answer" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                manage::update_answer(&conn, id, item, &*IMAGES_DIR).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_variant" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                manage::update_variant(&conn, id, item).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_bundle" => {

            let item: ganbare::models::AudioBundle = err_400!(serde_json::from_str(&text),
                                                              "Error decoding JSON");
            if item.id != id {
                return abort(400);
            }
            let updated_item = try_or!(
                audio::change_bundle_name(&conn, id, &item.listname).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_audio_file" => {

            let item = err_400!(serde_json::from_str(&text), "Error decoding JSON");

            let updated_item = try_or!(
                audio::update_file(&conn, id, &item).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_narrator" => {

            let item: ganbare::models::Narrator = err_400!(serde_json::from_str(&text),
                                                           "Error decoding JSON");
            if item.id != id {
                return abort(400);
            }
            let updated_item = try_or!(
                audio::update_narrator(&conn, &item).err_500()?,
                else return abort(404)
            );

            json = jsonify(&updated_item);
        }
        "update_event" => {

            let item: ganbare::models::UpdateEvent = err_400!(serde_json::from_str(&text),
                                                              "Couldn't parse the data!");

            if item.id != id {
                return abort(400);
            }
            let updated_item =
                try_or!(event::update_event(&conn, &item).err_500()?, else return abort(404));

            json = jsonify(&updated_item);
        }
        _ => return Err(internal_error("no such endpoint!")),
    }

    json.refresh_cookie(&sess)
}


pub fn post_question(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "editors")?;

    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    use ganbare::models::{UpdateQuestion, UpdateAnswer, NewQuizQuestion, NewAnswer};

    let (qq, aas): (UpdateQuestion, Vec<UpdateAnswer>) =
        serde_json::from_str(&text).map_err(|_| abort(400).unwrap_err())?;

    fn parse_qq(qq: &UpdateQuestion) -> Result<NewQuizQuestion> {
        let qq = NewQuizQuestion {
            skill_id: qq.skill_id.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
            q_name: qq.q_name
                .as_ref()
                .ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?
                .as_str(),
            q_explanation: qq.q_explanation
                .as_ref()
                .ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?
                .as_str(),
            question_text: qq.question_text
                .as_ref()
                .ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?
                .as_str(),
            skill_level: qq.skill_level.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
        };
        Ok(qq)
    }

    fn parse_aa(aa: &UpdateAnswer) -> Result<NewAnswer> {
        let aa = NewAnswer {
            question_id: aa.question_id.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
            a_audio_bundle: aa.a_audio_bundle.unwrap_or(None),
            q_audio_bundle: aa.q_audio_bundle
                .ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
            answer_text: aa.answer_text
                .as_ref()
                .ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?
                .as_str(),
        };
        Ok(aa)
    }

    let new_qq = parse_qq(&qq).map_err(|_| abort(400).unwrap_err())?;

    let mut new_aas = vec![];
    for aa in &aas {
        let new_aa = parse_aa(aa).map_err(|_| abort(400).unwrap_err())?;
        new_aas.push(new_aa);
    }

    let id = manage::post_question(&conn, new_qq, new_aas).err_500()?;

    let new_url = format!("/api/questions/{}", id);

    redirect(&new_url, 303).refresh_cookie(&sess)
}

pub fn post_exercise(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "editors")?;

    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    use ganbare::models::{UpdateExercise, UpdateExerciseVariant, NewExercise, ExerciseVariant};

    let (qq, aas): (UpdateExercise, Vec<UpdateExerciseVariant>) =
        err_400!(serde_json::from_str(&text), "Error when parsing the JSON.");

    fn parse_qq(qq: &UpdateExercise) -> Result<NewExercise> {
        let qq = NewExercise {
            skill_id: qq.skill_id.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
            skill_level: qq.skill_level.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
        };
        Ok(qq)
    }

    fn parse_aa(aa: &UpdateExerciseVariant) -> Result<ExerciseVariant> {
        let aa = ExerciseVariant {
            id: aa.id.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
            exercise_id: aa.exercise_id.ok_or_else(|| Error::from_kind(ErrorKind::FormParseError))?,
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
    let (conn, sess) = auth_user(req, "")?;

    let eventname = req.view_args
        .remove("eventname")
        .expect("Pencil guarantees that Line ID should exist as an arg.");
    let key = req.view_args.remove("key");
    let (event, _) = event::require_ongoing(&conn, &eventname, sess.user_id).err_401()?;

    let mut text = String::new();
    req.read_to_string(&mut text).err_500()?;

    event::save_userdata(&conn,
                         &event,
                         sess.user_id,
                         key.as_ref().map(|s| &**s),
                         &text).err_500()?;

    Response::from("OK.").refresh_cookie(&sess)
}

pub fn user(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "admins")?;

    let user_id = req.view_args
        .remove("user_id")
        .expect("Pencil guarantees that Line ID should exist as an arg.");
    let user_id =
        user_id.parse::<i32>().expect("Pencil guarantees that Line ID should be an integer.");

    let endpoint = req.endpoint().expect("Pencil guarantees this");
    let json = match endpoint.as_ref() {
        "add_group" => {

            let group_id = req.view_args
                .remove("group_id")
                .expect("Pencil guarantees that Line ID should exist as an arg.");
            let group_id = group_id.parse::<i32>()
                .expect("Pencil guarantees that Line ID should be an integer.");

            ganbare::user::join_user_group_by_id(&conn, user_id, group_id).err_500()?;
            jsonify(&())
        }
        "remove_group" => {

            let group_id = req.view_args
                .remove("group_id")
                .expect("Pencil guarantees that Line ID should exist as an arg.");
            let group_id = group_id.parse::<i32>()
                .expect("Pencil guarantees that Line ID should be an integer.");

            ganbare::user::remove_user_group_by_id(&conn, user_id, group_id).err_500()?;
            jsonify(&())
        }
        "set_metrics" => {

            use ganbare::models::UpdateUserMetrics;

            let mut text = String::new();
            req.read_to_string(&mut text).err_500()?;
            let metrics: UpdateUserMetrics = err_400!(serde_json::from_str(&text),
                                                      "Can't decode JSON: {:?}",
                                                      &text);

            if user_id != metrics.id {
                return Ok(bad_request("user id in the URL must be the same as in the JSON \
                                       content!"));
            }

            ganbare::user::set_metrics(&conn, &metrics).err_500()?;
            jsonify(&())
        }
        _ => return Err(internal_error("no such endpoint!")),
    };

    json.refresh_cookie(&sess)
}

pub fn post_useraudio(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;
    use std::fs;
    use rand::thread_rng;
    use rand::Rng;
    use rand::distributions::Alphanumeric;

    let event_name = req.view_args
        .remove("event_name")
        .expect("Pencil guarantees that Line ID should exist as an arg.");

    let cl = err_400!(req.headers().get::<ContentLength>(),
                      "Content-Length must be set!")
            .0;
    debug!("post_useraudio. Content-Length: {:?}", cl);

    if cl > 60_000 && event_name != "pretest_retelling" && event_name != "posttest_retelling" {
        return Ok(bad_request("Too big audio file! It must be under 60kB"));
    } else if cl > 180_000 {
        return Ok(bad_request("Too big audio file! It must be under 180kB"));
    }

    let (event, _) = event::require_ongoing(&conn, &event_name, sess.user_id).err_401()?;

    let mut new_path = USER_AUDIO_DIR.to_owned();
    let mut filename = "%FT%H-%M-%SZ".to_string();
    filename.extend(thread_rng().sample_iter(Alphanumeric).take(10));
    filename.push_str(".ogg");
    filename = time::strftime(&filename, &time::now()).unwrap();
    new_path.push(&filename);

    let mut file = fs::File::create(&new_path).err_500()?;
    io::copy(&mut req.take(cl), &mut file).err_500()?;

    let mut rec_number = event::get_userdata(&conn, &event, sess.user_id, "rec_number")
        .err_500()?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);

    rec_number += 1;

    let quiz_number = event::get_userdata(&conn, &event, sess.user_id, "quiz_number")
        .err_500()?
        .and_then(|d| d.data.parse::<usize>().ok())
        .unwrap_or(0);
    event::save_userdata(&conn,
                         &event,
                         sess.user_id,
                         Some("rec_number"),
                         &format!("{}", rec_number)).err_500()?;
    event::save_userdata(&conn,
                         &event,
                         sess.user_id,
                         Some(&format!("quiz_{}_rec_{}", quiz_number, rec_number)),
                         &filename).err_500()?;

    debug!("Saved user audio: {:?} with rec_number: {} and quiz_number: {}",
           filename,
           rec_number,
           quiz_number);

    jsonify(&(quiz_number, rec_number)).refresh_cookie(&sess)
}

pub fn get_useraudio(req: &mut Request) -> PencilResult {

    let (conn, sess) = auth_user(req, "")?;
    let endpoint = req.endpoint().expect("Pencil guarantees this");

    let event_name = req.view_args
        .remove("event_name")
        .expect("Pencil guarantees that event name should exist as an arg.");

    let (event, _) = event::require_ongoing(&conn, &event_name, sess.user_id).err_401()?;

    let (quiz_number, rec_number) = match endpoint.as_ref() {
        "get_useraudio" => {

            let quiz_number = err_400!(req.view_args.remove("quiz_number").and_then(|d| d.parse::<usize>().ok()),
                         "quiz_number must be specified");
            let rec_number = err_400!(req.view_args.remove("rec_number").and_then(|d| d.parse::<usize>().ok()),
                         "rec_number must be specified");
            (quiz_number, rec_number)
        }
        _ => unreachable!(),
    };

    let filename = try_or!(
        event::get_userdata(&conn,
                            &event,
                            sess.user_id,
                            &format!("quiz_{}_rec_{}", quiz_number, rec_number)
                        ).err_500()?,
        else {
            debug!("No userdata. rec_number: {}, quiz_number: {}", rec_number, quiz_number);
            return abort(404)
        }
    );
    debug!("Getting user audio. rec_number: {}, quiz_number: {}, filename: {}",
           rec_number,
           quiz_number,
           &filename.data);
    let mut file_path = USER_AUDIO_DIR.clone();
    file_path.push(&filename.data);
    use pencil::{PencilError, HTTPError};
    use hyper::header::{CacheControl, CacheDirective};
    use std::str::FromStr;
    send_file_range(file_path.to_str().expect("The path should be fully ASCII"),
                    mime::Mime::from_str("audio/ogg").unwrap(),
                    false,
                    req.headers().get())
            .set_static_cache()
            .refresh_cookie(&sess)
            .map(|mut resp| {
                     resp.headers.set(CacheControl(vec![CacheDirective::NoCache,
                                                        CacheDirective::NoStore]));
                     resp
                 })
            .map_err(|e| match e {
                         PencilError::PenHTTPError(HTTPError::NotFound) => {
                error!("Audio file not found? The audio file database/folder is borked? \
                        {:?}",
                       file_path);
                internal_error(e)
            }
                         _ => internal_error(e),
                     })
}


pub fn mic_check(req: &mut Request) -> PencilResult {
    let (_, sess) = auth_user(req, "")?;

    let endpoint = req.endpoint().expect("Pencil guarantees this");

    let random_token = err_400!(req.view_args
                                    .remove("random_token")
                                    .expect("Pencil guarantees that event name should exist as \
                                             an arg.")
                                    .parse::<u64>(),
                                "The token must be parseable to u64!");

    match endpoint.as_ref() {
        "mic_check_rec" => {
            let cl = err_400!(req.headers().get::<ContentLength>(),
                              "Content-Length must be set!")
                    .0;
            debug!("mic_check_rec with random token: {}. Content-Length: {:?}",
                   random_token,
                   cl);
            if cl > 60_000 {
                return Ok(bad_request("Too big audio file! It must be under 60kB"));
            }
            let mut audio = Vec::with_capacity(cl as usize);
            io::copy(&mut req.take(cl), &mut audio).err_500()?;

            debug!("mic_check_rec audio read into vec. Next: saving it into a temp storage");

            AUDIO_CACHE.insert(random_token, audio).err_500()?;

            debug!("mic_check_rec done with random token: {} ", random_token);
            jsonify(&()).refresh_cookie(&sess)
        }
        "mic_check_play" => {
            debug!("mic_check_play with random token: {}", random_token);
            use std::str::FromStr;

            let audio = err_400!(AUDIO_CACHE.get(&random_token).err_500()?,
                                 "No such audio clip!");

            let mut resp = Response::from(&audio[..]);
            let mime = mime::Mime::from_str("audio/ogg").unwrap();
            resp.headers.set::<hyper::header::ContentType>(hyper::header::ContentType(mime));
            resp.refresh_cookie(&sess)
        }
        _ => unreachable!(),
    }
}

pub fn new_retelling(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;
    let event_name = req.view_args
        .remove("event_name")
        .expect("Pencil guarantees that event name should exist as an arg.");
    let (event, _) = event::require_ongoing(&conn, &event_name, sess.user_id).err_401()?;

    let retelling = match event_name.as_ref() {
        "pretest_retelling" => {
            test::get_new_retelling_pretest(&conn, sess.user_id, &event).err_500()?
        }
        "posttest_retelling" => {
            test::get_new_retelling_posttest(&conn, sess.user_id, &event).err_500()?
        }
        _ => unreachable!(),
    };

    jsonify(&retelling).refresh_cookie(&sess)
}

pub fn next_retelling(req: &mut Request) -> PencilResult {
    let (conn, sess) = auth_user(req, "")?;
    let event_name = req.view_args
        .remove("event_name")
        .expect("Pencil guarantees that event name should exist as an arg.");
    let (event, _) = event::require_ongoing(&conn, &event_name, sess.user_id).err_401()?;

    let retelling = match event_name.as_ref() {
        "pretest_retelling" => {
            test::get_next_retelling_pretest(&conn, sess.user_id, &event).err_500()?
        }
        "posttest_retelling" => {
            test::get_next_retelling_posttest(&conn, sess.user_id, &event).err_500()?
        }
        _ => unreachable!(),
    };

    match retelling {
            Some(ref retelling) => jsonify(retelling),
            None => jsonify(&()),
        }
        .refresh_cookie(&sess)
}
