
use std;
use std::env;
use dotenv;
use std::net::{SocketAddr, ToSocketAddrs};
use ganbare::PgConnection;
use std::collections::BTreeMap;
use cookie::Cookie as CookiePair;
use pencil::{self, Request, Response, abort, PencilError, PencilResult, SetCookie, Cookie};
use ganbare::models::{User, Session};
use std::net::IpAddr;
use time;
use std::result::Result as StdResult;
use ganbare::errors::Result;
use rustc_serialize::base64::FromBase64;
use ganbare::db;
use ganbare::user;
use ganbare::session;
use ganbare::errors;
use std::path::PathBuf;
pub use try_map::{FallibleMapExt, FlipResultExt};
pub use std::time::{Instant, Duration};

lazy_static! {

    pub static ref DATABASE_URL : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_DATABASE_URL")
            .expect(
            "GANBARE_DATABASE_URL must be set (format: postgres://username:password@host/dbname)"
            )
    };

    pub static ref SITE_DOMAIN : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_SITE_DOMAIN")
            .expect(
             "GANBARE_SITE_DOMAIN: Set the site domain! (Without it, the cookies don't work.)"
            )
    };

    pub static ref SITE_LINK : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_SITE_LINK")
            .unwrap_or_else(|_|
                format!("http://{}:8081", env::var("GANBARE_SITE_DOMAIN")
                    .unwrap_or_else(|_| "".into()))
                )
    };

    pub static ref EMAIL_SERVER : SocketAddr = {
        dotenv::dotenv().ok();
        let binding = env::var("GANBARE_EMAIL_SERVER")
            .expect(
            "GANBARE_EMAIL_SERVER: Specify an outbound email server, like this: mail.yourisp.com:25"
            );
        binding.to_socket_addrs().expect("Format: domain:port").next().expect("Format: domain:port")
    };

    pub static ref EMAIL_SMTP_USERNAME : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_EMAIL_SMTP_USERNAME")
            .unwrap_or_else(|_| "".into())
        };

    pub static ref EMAIL_SMTP_PASSWORD : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_EMAIL_SMTP_PASSWORD")
        .unwrap_or_else(|_| "".into())
    };

    pub static ref EMAIL_DOMAIN : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_EMAIL_DOMAIN")
            .unwrap_or_else(|_|  env::var("GANBARE_SITE_DOMAIN").unwrap_or_else(|_| "".into()))
    };

    pub static ref EMAIL_ADDRESS : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_EMAIL_ADDRESS")
            .unwrap_or_else(|_| format!("support@{}", &*EMAIL_DOMAIN))
    };

    pub static ref EMAIL_NAME : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_EMAIL_NAME")
            .unwrap_or_else(|_|  "".into())
    };

    pub static ref SERVER_BINDING : SocketAddr = {
        dotenv::dotenv().ok();
        let binding = env::var("GANBARE_SERVER_BINDING")
            .unwrap_or_else(|_| "localhost:8080".into());
        binding.to_socket_addrs().expect("GANBARE_SERVER_BINDING: Format: domain:port").next()
            .expect("GANBARE_SERVER_BINDING: Format: domain:port")
    };

    pub static ref JQUERY_URL : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_JQUERY")
            .unwrap_or_else(|_| "/static/js/jquery.min.js".into())
    };

    pub static ref FONT_URL : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_FONT_URL")
            .unwrap_or_else(|_| "/static/fonts/default.css".into())
    };

    pub static ref AUDIO_DIR : PathBuf = {
        dotenv::dotenv().ok();
        PathBuf::from(env::var("GANBARE_AUDIO_DIR")
            .unwrap_or_else(|_| "audio".into()))
    };

    pub static ref USER_AUDIO_DIR : PathBuf = {
        dotenv::dotenv().ok();
        PathBuf::from(env::var("GANBARE_USER_AUDIO_DIR")
            .unwrap_or_else(|_| "user_audio".into()))
    };

    pub static ref IMAGES_DIR : PathBuf = {
        dotenv::dotenv().ok();
        PathBuf::from(env::var("GANBARE_IMAGES_DIR")
            .unwrap_or_else(|_| "images".into()))
    };

    pub static ref PARANOID : bool = {
        dotenv::dotenv().ok();
        env::var("GANBARE_PARANOID").map(|s| s.parse::<bool>().unwrap_or(true))
            .unwrap_or(true)
    };

    pub static ref RUNTIME_PEPPER : Vec<u8> = {
        dotenv::dotenv().ok();
        let pepper = env::var("GANBARE_RUNTIME_PEPPER")
            .expect(
                "Environmental variable GANBARE_RUNTIME_PEPPER must be set!\
                (format: 256-bit random value encoded as base64)"
            )
            .from_base64().expect(
                "Environmental variable GANBARE_RUNTIME_PEPPER isn't valid Base64!
            ");
        if pepper.len() != 32 {
            panic!("The value must be 256-bit, that is, 32 bytes long!")
        }
        pepper
    };

    pub static ref BUILD_NUMBER : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_BUILD_NUMBER")
            .unwrap_or_else(|_| "not set".into())
    };

    pub static ref CONTENT_SECURITY_POLICY : String = {
        dotenv::dotenv().ok();
        env::var("GANBARE_CONTENT_SECURITY_POLICY")
            .unwrap_or_else(|_|
                "default-src 'self'; \
                style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                font-src 'self' https://fonts.gstatic.com https://fonts.googleapis.com; \
                script-src 'self' 'unsafe-inline' https://ajax.googleapis.com".into()
            )
    };

}

pub fn db_connect() -> Result<PgConnection> {

    let start = Instant::now();
    let conn = db::connect(&*DATABASE_URL)?;
    let end = Instant::now();
    let lag = end.duration_since(start);
    if lag > std::time::Duration::from_millis(10) {
        debug!("Connecting to DB took {:?} ms",
               lag.subsec_nanos() / 1_000_000);
    }
    Ok(conn)
}


pub fn get_cookie(cookies: &Cookie) -> Option<&str> {
    for c in cookies.0.iter().map(String::as_str) {
        match CookiePair::parse(c) {
            Ok(c) => {
                if c.name() == "session_id" {
                    return Some(c.value().long_or_panic());
                }
            }
            Err(_) => return None,
        }
    }
    None
}

pub fn new_template_context() -> BTreeMap<String, String> {
    let mut ctx = BTreeMap::new();
    ctx.insert("title".to_string(), "akusento.ganba.re".to_string());
    ctx.insert("jquery_url".to_string(), JQUERY_URL.to_string());
    ctx.insert("font_stylesheet".to_string(), FONT_URL.to_string());
    ctx
}

pub fn get_user(conn: &PgConnection, req: &Request) -> Result<Option<(User, Session)>> {
    if let Some(sess_token) = req.cookies().and_then(get_cookie) {
        Ok(session::check(conn, sess_token, req.remote_addr().ip())?)
    } else {
        Ok(None)
    }
}

pub trait IntoIp {
    fn into_ip(self) -> IpAddr;
}

impl IntoIp for IpAddr {
    fn into_ip(self) -> IpAddr {
        self
    }
}

impl<'a, 'b, 'c> IntoIp for Request<'a, 'b, 'c> {
    fn into_ip(self) -> IpAddr {
        self.request.remote_addr.ip()
    }
}

impl<'r, 'a, 'b, 'c> IntoIp for &'r mut Request<'a, 'b, 'c> {
    fn into_ip(self) -> IpAddr {
        self.request.remote_addr.ip()
    }
}

impl<'r, 'a, 'b, 'c> IntoIp for &'r Request<'a, 'b, 'c> {
    fn into_ip(self) -> IpAddr {
        self.request.remote_addr.ip()
    }
}

pub trait CookieProcessor {
    fn refresh_cookie(self, &Session) -> PencilResult;
    fn expire_cookie(self) -> Self;
}

impl CookieProcessor for Response {
    fn refresh_cookie(mut self, sess: &Session) -> PencilResult {
        let cookie = CookiePair::build("session_id", session::to_hex(sess))
            .path("/")
            .http_only(true)
            .secure(*PARANOID)
            .domain(SITE_DOMAIN.as_str())
            .expires(time::now_utc() + time::Duration::weeks(2));
        self.set_cookie(SetCookie(vec![format!("{}", cookie.finish())]));
        Ok(self)
    }

    fn expire_cookie(mut self) -> Self {
        let cookie = CookiePair::build("session_id", "")
            .path("/")
            .domain(SITE_DOMAIN.as_str())
            .expires(time::at_utc(time::Timespec::new(0, 0)));
        self.set_cookie(SetCookie(vec![format!("{}", cookie.finish())]));
        self
    }
}

impl CookieProcessor for PencilResult {
    fn refresh_cookie(self, sess: &Session) -> PencilResult {
        self.and_then(|resp| resp.refresh_cookie(sess))
    }

    fn expire_cookie(self) -> Self {
        self.and_then(|resp| {
            Ok(<Response as CookieProcessor>::expire_cookie(resp))
        })
    }
}

macro_rules! try_or {
    ($t:expr , else $e:expr ) => {  match $t { Some(x) => x, None => { $e } };  }
}

pub fn internal_error<T: std::fmt::Debug>(err: T) -> PencilError {
    error!("{:?}", err);
    PencilError::PenHTTPError(pencil::http_errors::HTTPError::InternalServerError)
}

pub fn bad_request<T: ToString + std::fmt::Debug>(err_msg: T) -> Response {
    warn!("Error 400: Bad request. {:?}", err_msg.to_string());
    let body = err_msg.to_string();
    let mut resp = pencil::Response::new(body);
    resp.status_code = 400;
    resp
}

pub trait ResultExt<T> {
    fn err_500(self) -> StdResult<T, PencilError>;
    fn err_500_debug(self, user: &User, req: &Request) -> StdResult<T, PencilError>;
    fn err_401(self) -> StdResult<T, PencilError>;
}

impl<T, E: std::fmt::Debug> ResultExt<T> for StdResult<T, E> {
    fn err_500(self) -> StdResult<T, PencilError> {
        self.map_err(internal_error)
    }
    fn err_500_debug(self, user: &User, req: &Request) -> StdResult<T, PencilError> {
        self.map_err(|e| internal_error((e, user, req)))
    }
    fn err_401(self) -> StdResult<T, PencilError> {
        self.map_err(|_| PencilError::PenHTTPError(pencil::http_errors::HTTPError::Unauthorized))
    }
}

pub trait CarrierInternal<T, E>
    where E: std::fmt::Debug
{
    fn ok_or(self) -> std::result::Result<T, E>;
}

impl<T> CarrierInternal<T, errors::Error> for Option<T> {
    fn ok_or(self) -> std::result::Result<T, errors::Error> {
        match self {
            Some(a) => Ok(a),
            None => Err(errors::ErrorKind::NoneResult.into()),
        }
    }
}
impl<T, E> CarrierInternal<T, E> for std::result::Result<T, E>
    where E: std::fmt::Debug
{
    fn ok_or(self) -> std::result::Result<T, E> {
        match self {
            Ok(a) => Ok(a),
            Err(e) => Err(e),
        }
    }
}

macro_rules! err_400 {
    ($t:expr , $format_string:expr $(, $param:expr)* ) => {
        match CarrierInternal::ok_or($t) {
            Ok(a) => { a },
            Err(e) => {
                use std::error::Error;
                return Ok(bad_request(
                    format!(
                        concat!(
                            "<h1>HTTP 400 Bad Request {:?}: ", $format_string, "</h1>"
                        ), e.description() $(, $param)*
                    )
                ))
            },
        }
    }
}

#[cfg(debug_assertions)]
macro_rules! include_templates(
    ($app:ident, $temp_dir:expr, $($file:expr),*) => { {
        $app.template_folder = $temp_dir.to_string();
        $(
            $app.register_template($file);
        )*
        info!("Templates loaded.");
    } }
);

#[cfg(not(debug_assertions))]
macro_rules! include_templates(
    ($app:ident, $temp_dir:expr, $($file:expr),*) => { {
        let mut reg = $app.handlebars_registry
            .write()
            .expect("This is supposed to fail fast and hard.");
        $(
            reg.register_template_string($file, include_str!(
                concat!(env!("PWD"), "/", $temp_dir, "/", $file)).to_string()
            )
            .expect("This is supposed to fail fast and hard.");
        )*
    } }
);

#[cfg(feature="perf_trace")]
macro_rules! time_it {
    ($comment:expr , $code:expr) => {
        {
            let start = Instant::now();

            let res = $code;

            let end = Instant::now();
            let lag = end.duration_since(start);
            debug!("{}:{} time_it {} took {}s {}ms!",
                file!(),
                line!(),
                $comment,
                lag.as_secs(),
                lag.subsec_nanos()/1_000_000);
            res
        }
    };
    ($code:expr) => {
        {
            let start = Instant::now();

            let res = $code;

            let end = Instant::now();
            let lag = end.duration_since(start);
            debug!("{}:{} time_it took {}s {}ms!",
                file!(),
                line!(),
                lag.as_secs(),
                lag.subsec_nanos()/1_000_000);
            res
        }
    }
}

#[cfg(not(feature="perf_trace"))]
macro_rules! time_it {
    ($comment:expr , $code:expr) => {{ $code }};
    ($code:expr) => {{ $code }}
}

pub fn auth_user(req: &mut Request,
                 required_group: &str)
                 -> StdResult<(PgConnection, User, Session), PencilError> {

    time_it!{"authentication",
        match try_auth_user(req)? {
            Some((conn, user, sess)) => {
                if user::check_user_group(&conn, user.id, required_group).err_500()? {
                    Ok((conn, user, sess))
                } else {
                    Err(abort(401).unwrap_err()) // User doesn't belong in the required groups
                }
            }
            None => {
                Err(abort(401).unwrap_err()) // User isn't logged in
            }
        }
    }
}

pub fn try_auth_user(req: &mut Request)
                     -> StdResult<Option<(PgConnection, User, Session)>, PencilError> {

    let conn = db_connect().err_500()?;

    if let Some((user, sess)) = get_user(&conn, req).err_500()? {
        // User is logged in

        Ok(Some((conn, user, sess)))

    } else {
        // Not logged in
        Ok(None)
    }

}

/// Try and dereference required env vars for the `lazy_static!`
/// to run and check if the values are present.
pub fn check_env_vars() {
    let _ = &*DATABASE_URL;
    let _ = &*EMAIL_SERVER;
    let _ = &*SITE_DOMAIN;
}

pub fn do_login<I: IntoIp>(conn: &PgConnection,
                           email: &str,
                           plaintext_pw: &str,
                           ip: I)
                           -> StdResult<Option<(User, Session)>, PencilError> {
    debug!("Logging in user: {:?}", email);
    let user = try_or!(user::auth_user(&conn, email, plaintext_pw, &*RUNTIME_PEPPER).err_500()?,
            else return Ok(None));

    let sess = session::start(conn, &user, ip.into_ip()).err_500()?;

    Ok(Some((user, sess)))
}

pub fn do_logout(conn: &PgConnection, sess: &Session) -> StdResult<(), PencilError> {
    debug!("Logging out session: {:?}", sess);
    session::end(conn, sess).err_500()?;
    Ok(())
}

macro_rules! parse {
    ($expression:expr) => {
        $expression
            .map(String::to_string)
            .ok_or(Error::from_kind(ErrorKind::FormParseError))?;
    }
}


pub fn rate_limit<O, F: FnOnce() -> O>(pause_duration: Duration,
                                       random_max_millis: u64,
                                       function: F)
                                       -> O {
    use std::thread;
    use rand::{Rng, OsRng};
    let mut os_rng = OsRng::new().expect("If the OS RNG is not present, just crash.");

    // I THINK 0-5 ms of random duration is enough to mask all kinds of regularities
    // such as rounding artefacts etc.
    // (Apparently Linux and OS X have 1ms thread sleep granularity,
    // whereas Windows has something like 10-15ms.)
    #[cfg(target_os = "linux")]
    let randomized_duration = Duration::from_millis(os_rng.gen_range(0, random_max_millis));
    #[cfg(target_os = "macos")]
    let randomized_duration = Duration::from_millis(os_rng.gen_range(0, random_max_millis));
    #[cfg(target_os = "windows")]
    let randomized_duration = Duration::from_millis(os_rng.gen_range(0, random_max_millis * 10));

    let start_time = Instant::now();

    let result = function();

    let worked_duration = Instant::now() - start_time;

    if pause_duration > worked_duration {

        thread::sleep(pause_duration - worked_duration + randomized_duration);

    } else {
        // Oops, the work took more time than expected and we're leaking information!
        // At least we can try and fumble a bit.

        error!("rate limit: The work took more time than expected! We're leaking information!");
        thread::sleep(randomized_duration);

    }

    result
}
