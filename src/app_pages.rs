
use super::*;
use pencil::redirect;

pub fn hello(req: &mut Request) -> PencilResult {

    if let Some((conn, user, sess)) = try_auth_user(req).err_500()? {

        if let Some(event_redirect) = dispatch_events(req, &conn, &user, &sess)? {
            event_redirect
        } else {
            main_quiz(req, &conn, &user)
        }

    } else {
        return redirect("/login", 303)
    }
}

pub fn ok(req: &mut Request) -> PencilResult {

    let (conn, user, sess) = auth_user(req, "")?;

    let event_name = err_400!(req.form_mut().take("event_ok"), "Field event_ok is missing!");
    let _ = err_400!(ganbare::set_event_done(&conn, &event_name, &user).err_500()?, "Event \"{}\" doesn't exist!", &event_name);


    redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}

pub fn dispatch_events(req: &mut Request, conn: &PgConnection, user: &User, sess: &Session)
    -> StdResult<Option<PencilResult>, PencilError> {

    let event_redirect = if ! ganbare::is_event_done(conn, "welcome", &user).err_500()? {

        Some(redirect("/welcome", 303))

    } else if ! ganbare::is_event_done(conn, "survey", &user).err_500()? {

        Some(redirect("/survey", 303))

    } else { None };

    Ok(event_redirect.map(|redirect| redirect.map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))))
}

pub fn main_quiz(req: &mut Request, _: &PgConnection, _: &User) -> PencilResult { 
    let context = new_template_context();
    req.app.render_template("main.html", &context)
}

pub fn survey(req: &mut Request) -> PencilResult {
    let (conn, user, _) = auth_user(req, "")?;
    ganbare::initiate_event(&conn, "survey", &user).err_500()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), "survey".into());
    req.app.render_template("survey.html", &context)
}

pub fn welcome(req: &mut Request) -> PencilResult { 
    let (conn, user, _) = auth_user(req, "")?;
    ganbare::initiate_event(&conn, "welcome", &user).err_500()?;
    let mut context = new_template_context();
    context.insert("event_name".into(), "welcome".into());
    req.app.render_template("welcome.html", &context)
}

pub fn login_form(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some(_) = get_user(&conn, req).err_500()? {
        return redirect("/", 303)
    }
    if ! ganbare::is_installed(&conn).err_500()? {
        return redirect("/fresh_install", 303)
    }

    let context = new_template_context();

    req.app.render_template("hello.html", &context)
}

pub fn login_post(request: &mut Request) -> PencilResult {
    let app = request.app;
    let ip = request.request.remote_addr.ip();
    let login_form = request.form_mut();
    let email = login_form.take("email").unwrap_or_default();
    let plaintext_pw = login_form.take("password").unwrap_or_default();

    let conn = db_connect().err_500()?;

    match do_login(&email, &plaintext_pw, ip).err_500()? {
        Some((_, sess)) => {
            redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, ip) )
        },
        None => {
            warn!("Failed login.");
            let mut context = new_template_context();
            context.insert("authError".to_string(), "true".to_string());
            let result = app.render_template("hello.html", &context);
            result.map(|mut resp| {resp.status_code = 401; resp})
        },
    }
}

pub fn logout(request: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;
    if let Some(session_id) = request.cookies().and_then(get_cookie) {
        ganbare::end_session(&conn, &session_id).err_500()?;
    };

    redirect("/", 303).map(ResponseExt::expire_cookie)
}


pub fn confirm_form(request: &mut Request) -> PencilResult {

    let secret = err_400!(request.args().get("secret"), "secret");
    let conn = db_connect()
        .map_err(|e| internal_error(e) )?;
    let (email, _) = ganbare::check_pending_email_confirm(&conn, &secret)
        .err_500()?;

    let mut context = new_template_context();
    context.insert("email".to_string(), email);
    context.insert("secret".to_string(), secret.clone());

    request.app.render_template("confirm.html", &context)
}

pub fn confirm_post(req: &mut Request) -> PencilResult {
    req.load_form_data();
    let ip = req.request.remote_addr.ip();
    let conn = db_connect()
        .err_500()?;
    let secret = err_400!(req.args().get("secret"), "secret missing").clone();
    let password = err_400!(req.form().expect("form data loaded.").get("password"), "password missing");
    let user = match ganbare::complete_pending_email_confirm(&conn, &password, &secret, &*RUNTIME_PEPPER) {
        Ok(u) => u,
        Err(e) => match e.kind() {
            &ganbare::errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
            &ganbare::errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
            _ => return Err(internal_error(e)),
        }
    };

    match do_login(&user.email, &password, ip).err_500()? {
        Some((_, sess)) => {
            redirect("/", 303).map(|resp| resp.refresh_cookie(&conn, &sess, ip) )
        },
        None => { Err(internal_error(Error::from(ErrMsg("We just added the user, yet we can't login them in. A bug?".to_string())))) },
    }
}


pub fn change_password_form(req: &mut Request) -> PencilResult {

    let (conn, _, sess) = auth_user(req, "")?;

    let mut context = new_template_context();

    let password_changed = req.args_mut().take("password_changed")
        .and_then(|a| if a == "true" { Some(a) } else { None })
        .unwrap_or_else(|| "false".to_string());

    context.insert("password_changed".to_string(), password_changed);

    req.app.render_template("change_password.html", &context)
                    .map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()))
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

    match ganbare::auth_user(&conn, &user.email, &old_password, &*RUNTIME_PEPPER) {
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
            match ganbare::change_password(&conn, user.id, &new_password, &*RUNTIME_PEPPER) {
                Err(e) => match e.kind() {
                    &ganbare::errors::ErrorKind::PasswordTooShort => return Ok(bad_request("Password too short")),
                    &ganbare::errors::ErrorKind::PasswordTooLong => return Ok(bad_request("Password too long")),
                    _ => return Err(internal_error(e)),
                },
                _ => (),
            };
        },
    };

    redirect("/change_password?password_changed=true", 303).map(|resp| resp.refresh_cookie(&conn, &sess, req.remote_addr().ip()) )
}
