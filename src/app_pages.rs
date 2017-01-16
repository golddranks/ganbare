
use super::*;
use pencil::redirect;
use ganbare::errors;
use ganbare::event;
use ganbare::user;
use ganbare::email;

fn dispatch_events(conn: &PgConnection, user: &User)
    -> StdResult<Option<PencilResult>, PencilError> {

    let event = match event::dispatch_event(conn, user.id).err_500()? {
        Some(e) => e,
        None => return Ok(None),
    };

    event::initiate(conn, &*event.name, user).err_500()?;
    
    let redirect = match &*event.name {
        "welcome" => redirect("/welcome", 303),
        "agreement" => redirect("/agreement", 303),
        "info" => redirect("/info", 303),
        "survey" => redirect("/survey", 303),
        "pretest_info" => redirect("/pretest_info", 303),
        "pretest" => redirect("/pretest", 303),
        "pretest_retelling" => redirect("/pretest_retelling", 303),
        "pretest_done" => redirect("/pretest_done", 303),
        "sorting_ceremony" => redirect("/sorting_ceremony", 303),
        "posttest_info" => redirect("/posttest_info", 303),
        "posttest" => redirect("/posttest", 303),
        "posttest_retelling" => redirect("/posttest_retelling", 303),
        "posttest_done" => redirect("/posttest_done", 303),
        ename => return Err(internal_error(&format!("I don't know how to handle event {}!", ename))),
    };

    Ok(Some(redirect))
}

fn main_quiz(req: &mut Request, conn: &PgConnection, user: &User) -> PencilResult { 
    let mut context = new_template_context();

    if ! user::check_user_group(conn, user.id, "input_group").err_500()? && ! user::check_user_group(conn, user.id, "output_group").err_500()? {
        context.insert("alert_msg".into(), "Et kuulu mihinkään harjoitusryhmään!".into());
    }

    req.app
        .render_template("main.html", &context)
}

pub fn hello(req: &mut Request) -> PencilResult {

    if let Some((conn, user, sess)) = try_auth_user(req).err_500()? {

        if let Some(event_redirect) = dispatch_events(&conn, &user)? {
            event_redirect
        } else {
            main_quiz(req, &conn, &user)

        }.refresh_cookie(&sess)

    } else {
        redirect("/login", 303)
    }
}

pub fn ok(req: &mut Request) -> PencilResult {

    let (conn, user, sess) = auth_user(req, "")?;

    let event_name = err_400!(req.form_mut().take("event_ok"), "Field event_ok is missing!");
    let _ = err_400!(event::set_done(&conn, &event_name, &user).err_500()?, "Event \"{}\" doesn't exist!", &event_name);


    redirect("/", 303).refresh_cookie(&sess)
}

pub fn survey(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;
    let (event, _) = event::require_ongoing(&conn, "survey", &user).err_401()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), "survey".into());
    let answered_questions = event::get_userdata(&conn, &event, &user, "answered_questions")
        .err_500()?
        .map(|d| d.data)
        .unwrap_or_else(|| "".to_string());

    context.insert("answered_questions".into(), answered_questions);
    req.app
        .render_template("survey.html", &context)
        .refresh_cookie(&sess)
}

pub fn text_pages(req: &mut Request) -> PencilResult { 
    let (conn, user, sess) = auth_user(req, "")?;

    let endpoint_string = req.endpoint().expect("Pencil guarantees that this is always set.");
    let endpoint = endpoint_string.as_ref();

    event::require_ongoing(&conn, endpoint, &user).err_401()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), endpoint.into());

    let mut template = endpoint.to_owned();
    template.push_str(".html");
    req.app
        .render_template(&template, &context)
        .refresh_cookie(&sess)
}

pub fn pre_post_test(req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "subjects")?;

    let mut context = new_template_context();

    match req.endpoint().as_ref().map(|s| &**s) {
        Some("pretest") => {
            if event::is_ongoing(&conn, "pretest", &user).err_500()?.is_some() {
                context.insert("testing".into(), "true".into());
            } else {
                return redirect("/", 303).refresh_cookie(&sess);
            }
        },
        Some("posttest") => {
            if event::is_ongoing(&conn, "posttest", &user).err_500()?.is_some() {
                context.insert("testing".into(), "true".into());
            } else {
                return redirect("/", 303).refresh_cookie(&sess);
            }
        },
        _ => unreachable!(),
    }

    req.app.render_template("main.html", &context)
        .refresh_cookie(&sess)
}

pub fn sorting_ceremony(req: &mut Request) -> PencilResult {
    use rand::{Rng, thread_rng};

    let (conn, user, sess) = auth_user(req, "ready_for_sorting")?;

    let (_, _) = event::require_ongoing(&conn, "sorting_ceremony", &user).err_401()?;

    let group_name = if user::check_user_group(&conn, user.id, "japani1").err_500()? { "japani1" }
        else if user::check_user_group(&conn, user.id, "japani2").err_500()? { "japani2" }
        else if user::check_user_group(&conn, user.id, "japani3").err_500()? { "japani3" }
        else if user::check_user_group(&conn, user.id, "japani4").err_500()? { "japani4" }
        else { "ready_for_sorting" };

    if user::remove_user_group_by_name(&conn, user.id, "input_group").err_500()? { debug!("Removed old input_group"); };
    if user::remove_user_group_by_name(&conn, user.id, "output_group").err_500()? { debug!("Removed old output_group"); };

    let mut membership = {
        let subjects_size = user::group_size(&conn, group_name).err_500()?;
        let quota = subjects_size/2 + subjects_size%2;
        let s_input_size = user::group_intersection_size(&conn, group_name, "input_group").err_500()?;
        let s_output_size = user::group_intersection_size(&conn, group_name, "output_group").err_500()?;
        let sort_to_input: bool = if s_input_size < quota && s_output_size < quota {
            thread_rng().gen::<bool>()
        } else if s_input_size >= quota {
            false
        } else if s_output_size >= quota {
            true
        } else { unreachable!() };

        if sort_to_input {
            user::join_user_group_by_name(&conn, &user, "input_group").err_500()?
        } else {
            user::join_user_group_by_name(&conn, &user, "output_group").err_500()?
        }
    };
    use ganbare::SaveChangesDsl;
    membership.anonymous = true;
    let _: ganbare::models::GroupMembership = membership.save_changes(&conn).err_500()?;
    
    event::set_done(&conn, "sorting_ceremony", &user).err_500()?;

    return redirect("/", 303)
        .refresh_cookie(&sess)
}

pub fn retelling (req: &mut Request) -> PencilResult {
    let (conn, user, sess) = auth_user(req, "")?;

    let event_name;

    match req.endpoint().as_ref().map(|s| &**s) {
        Some("pretest_retelling") => {
            event_name = "pretest_retelling";
        },
        Some("posttest_retelling") => {
            event_name = "posttest_retelling";
        },
        _ => unreachable!(),
    }

    let (_, _) = event::require_ongoing(&conn, &event_name, &user).err_401()?;

    let mut context = new_template_context();
    context.insert("testing".into(), "true".into());
    context.insert("event_name".into(), event_name.into());
    
    req.app.render_template("retelling.html", &context)
        .refresh_cookie(&sess)
}

pub fn login_form(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some((_, sess)) = get_user(&conn, req).err_500()? {
        return redirect("/", 303).refresh_cookie(&sess)
    }
    if ! ganbare::db::is_installed(&conn).err_500()? {
        return redirect("/fresh_install", 303)
    }

    let email = req.args().get("email").map(|s|&**s).unwrap_or_else(|| "").to_string();
    let mut context = new_template_context();
    context.insert("email".into(), email);

    req.app.render_template("hello.html", &context)
}

pub fn login_post(req: &mut Request) -> PencilResult {

    rate_limit(Duration::from_millis(1000), 20, || {

    let app = req.app;
    let ip = req.request.remote_addr.ip();
    let email = req.form_mut().take("email").unwrap_or_default();
    let plaintext_pw = req.form_mut().take("password").unwrap_or_default();

    let conn = db_connect().err_500()?;

    if let Some((_, old_sess)) = get_user(&conn, &*req).err_500()? {
        do_logout(&conn, &old_sess).err_500()?;
    }

    match do_login(&conn, &email, &plaintext_pw, ip).err_500()? {
        Some((_, sess)) => {
            redirect("/", 303).refresh_cookie(&sess)
        },
        None => {
            warn!("Failed login.");
            let mut context = new_template_context();
            context.insert("email".to_string(), email);
            context.insert("authError".to_string(), "true".to_string());
            let result = app.render_template("hello.html", &context);
            result.map(|mut resp| {resp.status_code = 401; resp})
        },
    }

    })
}

pub fn logout(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some((_, old_sess)) = get_user(&conn, &*req).err_500()? {
        do_logout(&conn, &old_sess).err_500()?;
    }
    return redirect("/", 303).expire_cookie()
}


pub fn confirm_form(request: &mut Request) -> PencilResult {

    rate_limit(Duration::from_millis(2000), 20, || {

    let secret = err_400!(request.args().get("secret"), "secret");
    let conn = db_connect().err_500()?;
    let email = match email::check_pending_email_confirm(&conn, &secret).err_500()? {
        Some((email, _)) => email,
        None => return redirect("/", 303),
    };

    let mut context = new_template_context();
    context.insert("email".to_string(), email);
    context.insert("secret".to_string(), secret.clone());

    request.app.render_template("confirm.html", &context)

    })
}

pub fn confirm_post(req: &mut Request) -> PencilResult {

    rate_limit(Duration::from_millis(2000), 20, || {

    req.load_form_data();
    let conn = db_connect().err_500()?;
    let secret = err_400!(req.args().get("secret"), "secret missing").clone();
    let email = err_400!(req.form().expect("form data loaded.").get("email"), "email field missing");
    let password = err_400!(req.form().expect("form data loaded.").get("password"), "password missing");
    let user = match email::complete_pending_email_confirm(&conn, &password, &secret, &*RUNTIME_PEPPER) {
        Ok(u) => u,
        Err(e) => match e.kind() {
            &errors::ErrorKind::NoSuchSess => {
                if user::get_user_by_email(&conn, email).err_500()?.is_some() {
                    return Ok(bad_request("The user account already exists?"))
                } else {
                    return Ok(bad_request(""))
                }
            },
            &errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
            &errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
            _ => return Err(internal_error(e)),
        }
    };

    let conn = db_connect().err_500()?;

    if let Some((_, old_sess)) = get_user(&conn, &*req).err_500()? {
        do_logout(&conn, &old_sess).err_500()?;
    }

    match do_login(&conn, &user.email.expect("The email address was just proven to exits."), &password, &*req).err_500()? {
        Some((_, sess)) =>
            redirect("/", 303).refresh_cookie(&sess),
        None => 
            Err(internal_error(Error::from(ErrMsg("We just added the user, yet we can't login them in. A bug?".to_string())))),
    }
    })
}


pub fn change_password_form(req: &mut Request) -> PencilResult {

    let (_, _, sess) = auth_user(req, "")?;

    let mut context = new_template_context();

    let password_changed = req.args_mut().take("password_changed")
        .and_then(|a| if a == "true" { Some(a) } else { None })
        .unwrap_or_else(|| "false".to_string());

    context.insert("password_changed".to_string(), password_changed);

    req.app
        .render_template("change_password.html", &context)
        .refresh_cookie(&sess)
}

pub fn password_reset_success(req: &mut Request) -> PencilResult {
    let (_, _, sess) = auth_user(req, "")?;
    let mut context = new_template_context();
    context.insert("changed".into(), "changed".into());
    req.app
        .render_template("reset_password.html", &context)
        .refresh_cookie(&sess)
}

pub fn confirm_password_reset_form(req: &mut Request) -> PencilResult {

    rate_limit(Duration::from_millis(2000), 20, || {

    let secret = err_400!(req.args_mut().take("secret"), "secret");
    let changed = req.args_mut().take("changed");
    let conn = db_connect().err_500()?;
    let email = match user::check_password_reset(&conn, &secret).err_500()? {
        Some((secret, _)) => secret.email,
        None => return redirect("/", 303),
    };

    let mut context = new_template_context();
    context.insert("email".into(), email);
    context.insert("secret".into(), secret);
    if let Some(changed) = changed {
        context.insert("changed".into(), changed);
    }

    req.app
        .render_template("reset_password.html", &context)
    
    })
}

pub fn confirm_password_reset_post(req: &mut Request) -> PencilResult {

    rate_limit(Duration::from_millis(2000), 20, || {

    let secret = err_400!(req.form_mut().take("secret"), "secret's missing");
    let password = err_400!(req.form_mut().take("new_password"), "password's missing");
    let new_password_check = err_400!(req.form_mut().take("new_password_check"), "password's missing");
    if password != new_password_check {
        return Ok(bad_request("Password and password check don't match!"));
    }

    let conn = db_connect().err_500()?;

    let (secret, user) = match user::check_password_reset(&conn, &secret).err_500()? {
        Some((secret, user)) => (secret, user),
        None => return redirect("/", 303),
    };

    user::invalidate_password_reset(&conn, &secret).err_500()?;

    match user::change_password(&conn, user.id, &password, &*RUNTIME_PEPPER) {
        Err(e) => match e.kind() {
            &errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
            &errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
            _ => return Err(internal_error(e)),
        },
        _ => (),
    };

    if let Some((_, old_sess)) = get_user(&conn, &*req).err_500()? {
        do_logout(&conn, &old_sess).err_500()?;
    }

    match do_login(&conn, &secret.email, &password, &*req).err_500()? {
        Some((_, sess)) =>
            redirect("/reset_password?changed=true", 303).refresh_cookie(&sess),
        None => 
            Err(internal_error(Error::from(ErrMsg("We just successfully changed password, yet we can't login them in. A bug?".to_string())))),
    }

    })
}

pub fn pw_reset_email_form(req: &mut Request) -> PencilResult {
    let email = req.args_mut().take("email").unwrap_or_else(|| "".into());
    let sent = req.args_mut().take("sent");
    let mut context = new_template_context();
    if sent.is_none() {
        context.insert("show_form".to_string(), "show_form".into());
    }
    context.insert("email".to_string(), email);
    context.insert("sent".to_string(), sent.unwrap_or_else(|| "".into()));

    req.app.render_template("send_pw_reset_email.html", &context)
}


pub fn send_pw_reset_email(req: &mut Request) -> PencilResult {

    fn parse_form(req: &mut Request) -> Result<String> {
        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        Ok(parse!(form.get("email")))
    }

    let conn = db_connect().err_500()?;

    let user_email = err_400!(parse_form(req), "invalid form data");

    match user::send_pw_change_email(&conn, &user_email) {
        Ok(secret) => {
            email::send_pw_reset_email(&secret, &*EMAIL_SERVER, &*EMAIL_SMTP_USERNAME, &*EMAIL_SMTP_PASSWORD, &*SITE_DOMAIN, &*SITE_LINK,
                &**req.app.handlebars_registry.read().expect("The registry is basically read-only after startup."),
                (&*EMAIL_ADDRESS, &*EMAIL_NAME)).err_500()?;
            redirect("/send_password_reset_email?sent=true", 303)
        },
        Err(Error(ErrorKind::NoSuchUser(_), _)) => {
            warn!("Trying to reset the password of non-existent address!");
            let mut context = new_template_context();
            context.insert("error".to_string(), "No such e-mail address :(".into());
            context.insert("show_form".to_string(), "show_form".into());
            req.app.render_template("send_pw_reset_email.html", &context).map(|mut r| { r.status_code = 400; r })
        },
        Err(Error(ErrorKind::RateLimitExceeded, _)) => {
            warn!("Someone is sending multiple password request requests per day!");
            let mut context = new_template_context();
            context.insert("error".to_string(), "Rate limit exceeded :(".into());
            req.app.render_template("send_pw_reset_email.html", &context).map(|mut r| { r.status_code = 429; r })
        },
        Err(e) => Err(internal_error(e)),
    }
}


pub fn change_password(req: &mut Request) -> PencilResult {

    fn parse_form(req: &mut Request) -> Result<(String, String)> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");

        let old_password = parse!(form.get("old_password"));
        let new_password = parse!(form.get("new_password"));
        if new_password != parse!(form.get("new_password_check")) {
            return Err("New passwords don't match!".into());
        }

        Ok((old_password, new_password))
    }

    let (conn, user, sess) = auth_user(req, "")?;

    let (old_password, new_password) = err_400!(parse_form(req), "invalid form data");

    let user_email = match user.email {
        Some(email) => email,
        None => return Ok(bad_request("User account is deactivated!? Cannot change password.")),
    };

    match user::auth_user(&conn, &user_email, &old_password, &*RUNTIME_PEPPER) {
        Err(e) => return match e.kind() {
            &ErrorKind::AuthError => {
                let mut context = new_template_context();
                context.insert("authError".to_string(), "true".to_string());

                req.app.render_template("change_password.html", &context)
                    .map(|mut resp| {resp.status_code = 401; resp})
            },
            _ => Err(internal_error(e)),
        },
        Ok(_) => {
            match user::change_password(&conn, user.id, &new_password, &*RUNTIME_PEPPER) {
                Err(e) => match e.kind() {
                    &errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
                    &errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
                    _ => return Err(internal_error(e)),
                },
                _ => (),
            };
        },
    };

    redirect("/change_password?password_changed=true", 303).refresh_cookie(&sess)
}

pub fn join_form(req: &mut Request) -> PencilResult {

    let context = new_template_context();

    req.app
        .render_template("join.html", &context)
}

pub fn join_post(req: &mut Request) -> PencilResult {

    let context = new_template_context();

    req.app
        .render_template("join.html", &context)
}
