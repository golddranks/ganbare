
use super::*;
use pencil::redirect;
use pencil::abort;
use ganbare::user;
use ganbare::manage;

pub fn fresh_install_form(req: &mut Request) -> PencilResult {
    let conn = ganbare::db::connect(&*DATABASE_URL).err_500()?;
    if ganbare::db::is_installed(&conn).err_500()? { return abort(401) };
    let context = new_template_context();
    req.app.render_template("fresh_install.html", &context)
}

pub fn fresh_install_post(req: &mut Request) -> PencilResult {
    req.load_form_data();
    let form = req.form().expect("Form data loaded");
    let email = err_400!(form.get("email"), "email missing");
    let new_password = err_400!(form.get("new_password"), "new_password missing");
    let new_password_check = err_400!(form.get("new_password_check"), "new_password_check missing");
    if new_password != new_password_check { return Ok(bad_request("passwords don't match")) };

    let conn = ganbare::db::connect(&*DATABASE_URL).err_500()?;
    if ganbare::db::is_installed(&conn).err_500()? { return abort(401) };

    let user = user::add_user(&conn, &email, &new_password, &*RUNTIME_PEPPER).err_500()?;
    user::join_user_group_by_name(&conn, &user, "admins").err_500()?;
    user::join_user_group_by_name(&conn, &user, "editors").err_500()?;


    match do_login(&user.email, &new_password, &*req).err_500()? {
        Some((_, mut sess)) => {
            let mut context = new_template_context();
            context.insert("install_success".into(), "success".into());
            req.app
                .render_template("fresh_install.html", &context)
                .refresh_cookie(&conn, &mut sess, &*req)
        },
        None => { Err(internal_error(Error::from(ErrMsg("We just added the user, yet we can't login them in. A bug?".to_string())))) },
    }
}

pub fn manage(req: &mut Request) -> PencilResult {
    let conn = db_connect().err_500()?;

    let (user, mut sess) = get_user(&conn, req).err_500()?
        .ok_or_else(|| abort(401).unwrap_err() )?; // Unauthorized

    if ! user::check_user_group(&conn, &user, "editors").err_500()?
        { return abort(401); }

    let context = new_template_context();

    req.app
        .render_template("manage.html", &context)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_quiz_form(req: &mut Request) -> PencilResult {

    let (conn, _, mut sess) = auth_user(req, "editors")?;

    let context = new_template_context();

    req.app
        .render_template("add_quiz.html", &context)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_quiz_post(req: &mut Request) -> PencilResult  {

    fn parse_form(req: &mut Request) -> Result<(manage::NewQuestion, Vec<manage::Fieldset>)> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let files = req.files().expect("Form data should be loaded!");;

        let lowest_fieldset = str::parse::<i32>(&parse!(form.get("lowest_fieldset")))?;
        if lowest_fieldset > 10 { return Err(ErrorKind::FormParseError.to_err()); }

        let q_name = parse!(form.get("name"));
        let q_explanation = parse!(form.get("explanation"));
        let question_text = parse!(form.get("question_text"));
        let skill_nugget = parse!(form.get("skill_nugget"));

        let mut fieldsets = Vec::with_capacity(lowest_fieldset as usize);
        for i in 1...lowest_fieldset {

            let q_variations = str::parse::<i32>(&parse!(form.get(&format!("choice_{}_q_variations", i))))?;
            if lowest_fieldset > 100 { return Err(ErrorKind::FormParseError.to_err()); }

            let mut q_variants = Vec::with_capacity(q_variations as usize);
            for v in 1...q_variations {
                if let Some(file) = files.get(&format!("choice_{}_q_variant_{}", i, v)) {
                    if file.size.expect("Size should've been parsed at this phase.") == 0 {
                        continue; // Don't save files with size 0;
                    }
                    let mut file = file.clone();
                    file.do_not_delete_on_drop();
                    q_variants.push(
                        (file.path.clone(),
                        file.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                        file.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                    );
                }
            }
            let answer_audio = files.get(&format!("choice_{}_answer_audio", i));
            let answer_audio_path;
            if let Some(path) = answer_audio {
                if path.size.expect("Size should've been parsed at this phase.") == 0 {
                    answer_audio_path = None;
                } else {
                    let mut cloned_path = path.clone();
                    cloned_path.do_not_delete_on_drop();
                    answer_audio_path = Some(
                        (cloned_path.path.clone(),
                        cloned_path.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                        cloned_path.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                    )
                }
            } else {
                answer_audio_path = None;
            };

            let answer_text = parse!(form.get(&format!("choice_{}_answer_text", i)));
            let fields = manage::Fieldset {q_variants: q_variants, answer_audio: answer_audio_path, answer_text: answer_text};
            fieldsets.push(fields);
        }

        Ok((manage::NewQuestion{q_name, q_explanation, question_text, skill_nugget}, fieldsets))
    }

    let (conn, _, mut sess) = auth_user(req, "editors")?;

    let form = parse_form(&mut *req).map_err(|ee| { error!("{:?}", ee); abort(400).unwrap_err()})?;
    let result = manage::create_quiz(&conn, form.0, form.1);
    result.map_err(|e| match e.kind() {
        &ErrorKind::FormParseError => abort(400).unwrap_err(),
        _ => abort(500).unwrap_err(),
    })?;

    redirect("/add_quiz", 303)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_word_form(req: &mut Request) -> PencilResult {
    let (conn, _, mut sess) = auth_user(req, "editors")?;

    let context = new_template_context();

    req.app
        .render_template("add_word.html", &context)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_word_post(req: &mut Request) -> PencilResult  {

    fn parse_form(req: &mut Request) -> Result<manage::NewWordFromStrings> {

        req.load_form_data();
        let form = req.form().expect("Form data should be loaded!");
        let uploaded_files = req.files().expect("Form data should be loaded!");

        let num_variants = str::parse::<i32>(&parse!(form.get("audio_variations")))?;
        if num_variants > 20 { return Err(ErrorKind::FormParseError.to_err()); }

        let word = parse!(form.get("word"));
        let explanation = parse!(form.get("explanation"));
        let nugget = parse!(form.get("skill_nugget"));

        let mut files = Vec::with_capacity(num_variants as usize);
        for v in 1...num_variants {
            if let Some(file) = uploaded_files.get(&format!("audio_variant_{}", v)) {
                if file.size.expect("Size should've been parsed at this phase.") == 0 {
                    continue; // Don't save files with size 0;
                }
                let mut file = file.clone();
                file.do_not_delete_on_drop();
                files.push(
                    (file.path.clone(),
                    file.filename().map_err(|_| ErrorKind::FormParseError.to_err())?,
                    file.content_type().ok_or(ErrorKind::FormParseError.to_err())?)
                );
            }
        }

        Ok(manage::NewWordFromStrings{word, explanation, narrator: "".into(), nugget, files})
    }

    let (conn, _, mut sess) = auth_user(req, "editors")?;

    let word = parse_form(req)
            .map_err(|_| abort(400).unwrap_err())?;

    manage::create_word(&conn, word).err_500()?;
    
    redirect("/add_word", 303)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_users_form(req: &mut Request) -> PencilResult {

    let (conn, _, mut sess) = auth_user(req, "admins")?;

    let context = new_template_context();
    req.app
        .render_template("add_users.html", &context)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}

pub fn add_users(req: &mut Request) -> PencilResult {
    let (conn, _, mut sess) = auth_user(req, "admins")?;

    req.load_form_data();
    let form = req.form().expect("The form data is loaded.");
    let emails = err_400!(form.get("emailList"),"emailList missing?");
    for row in emails.split("\n") {
        let mut fields = row.split_whitespace();
        let email = err_400!(fields.next(), "email field missing?");
        let mut groups = vec![];
        for field in fields {
            groups.push(try_or!(user::get_group(&conn, &field.to_lowercase())
                .err_500()?, else return abort(400)).id);
        }
        let secret = ganbare::email::add_pending_email_confirm(&conn, email, groups.as_ref())
            .err_500()?;
        ganbare::email::send_confirmation(email, &secret, &*EMAIL_SERVER, &*EMAIL_DOMAIN, &*SITE_DOMAIN, &**req.app.handlebars_registry.read()
                .expect("The registry is basically read-only after startup."))
            .err_500()?;
    }

    let context = new_template_context();
    req.app
        .render_template("add_users.html", &context)
        .refresh_cookie(&conn, &mut sess, req.remote_addr().ip())
}
