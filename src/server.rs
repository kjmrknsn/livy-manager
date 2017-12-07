use cmd_args::CmdArgs;
use config::Config;
use frontend::html::index::INDEX;
use frontend::html::login::LOGIN;
use iron::BeforeMiddleware;
use iron::headers::{CacheControl, CacheDirective, Connection, ContentType, Headers, Location, SetCookie};
use iron::mime;
use iron::mime::{Attr, Mime, TopLevel, SubLevel};
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use iron::status::Status;
use iron::Timeouts;
use iron::typemap::Key;
use ldap;
use livy::client::Client;
use params;
use params::Params;
use persistent::{Read, State};
use router::Router;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Debug};
use time;
use time::Duration;
use uuid::Uuid;

const COOKIE_NAME: &'static str = "_lmsid";

pub fn run() {
    let args = CmdArgs::new();

    if args.print_version {
        println!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let conf = Config::from(&args.conf_path);
    let user_sessions = UserSessions::new();

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/login", login, "login");
    router.post("/login", auth, "auth");
    router.get("/logout", logout, "logout");
    router.get("/api/user_session", get_user_session, "get_user_session");
    router.get("/api/sessions", get_sessions, "get_sessions");
    router.delete("/api/sessions/:id", kill_session, "kill_session");

    eprintln!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
    eprintln!("Listening on {}.", conf.http.addr);

    let mut chain = Chain::new(router);
    chain.link(Read::<Config>::both(conf.clone()));
    chain.link(State::<UserSessions>::both(user_sessions));
    chain.link_before(UserSessionBeforeMiddleware);

    let iron = Iron {
        handler: chain,
        timeouts: Timeouts::default(),
        threads: conf.http.num_threads,
    };
    iron.http(conf.http.addr).unwrap();
}

fn index(req: &mut Request) -> IronResult<Response> {
    if auth_required(req) && req.extensions.get::<UserSessionBeforeMiddleware>().is_none() {
        return Ok(redirect(status::TemporaryRedirect, "/login", None));
    }

    Ok(response(status::Ok, INDEX, text_html()))
}

fn login(req: &mut Request) -> IronResult<Response> {
    if !auth_required(req) {
        return Ok(redirect(status::TemporaryRedirect, "/", None));
    }

    if req.extensions.get::<UserSessionBeforeMiddleware>().is_some() {
        return Ok(redirect(status::TemporaryRedirect, "/", get_uuid(&req.headers).as_ref().map(String::as_str)));
    }

    Ok(response(status::Ok, LOGIN, text_html()))
}

fn auth(req: &mut Request) -> IronResult<Response> {
    if !auth_required(req) {
        return Ok(redirect(status::SeeOther, "/", None));
    }

    if req.extensions.get::<UserSessionBeforeMiddleware>().is_some() {
        return Ok(redirect(status::SeeOther, "/", get_uuid(&req.headers).as_ref().map(String::as_str)));
    }

    let params = match req.get_ref::<Params>() {
        Ok(params) => params.clone(),
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::BadRequest)),
    };

    match (params.find(&["uid"]), params.find(&["password"])) {
        (Some(&params::Value::String(ref uid)), Some(&params::Value::String(ref password))) => {
            let arc = req.get::<Read<Config>>().unwrap();
            let conf = match arc.as_ref().ldap.clone() {
                Some(conf) => conf,
                None => return Err(IronError::new(StringError("invalid request".to_string()), status::BadRequest))
            };

            match ldap::auth(&conf, uid.as_str(), password.as_str()) {
                Ok(user_session) => {
                    let uuid = Uuid::new_v4().to_string();
                    let arc = req.get::<State<UserSessions>>().unwrap();
                    let lock = arc.as_ref();
                    let mut user_sessions = lock.write().unwrap();
                    if user_sessions.map.contains_key(&uuid) {
                        return Ok(redirect(status::SeeOther, "/login?result=failed", None));
                    }
                    user_sessions.map.insert(uuid.clone(), user_session);
                    return Ok(redirect(status::SeeOther, "/", Some(&uuid)));
                },
                Err(_) => Ok(redirect(status::SeeOther, "/login?result=failed", None))
            }
        },
        _ => Err(IronError::new(StringError("invalid parameters".to_string()), status::BadRequest)),
    }
}

fn logout(req: &mut Request) -> IronResult<Response> {
    if !auth_required(req) {
        return Ok(redirect(status::TemporaryRedirect, "/", None));
    }

    if let (Some(_), Some(uuid)) = (req.extensions.get::<UserSessionBeforeMiddleware>(), get_uuid(&req.headers)) {
        let arc = req.get::<State<UserSessions>>().unwrap();
        let lock = arc.as_ref();
        let mut user_sessions = lock.write().unwrap();
        user_sessions.map.remove(&uuid);
    }

    Ok(redirect(status::TemporaryRedirect, "/login", None))
}

fn get_user_session(req: &mut Request) -> IronResult<Response> {
    let user_session = req.extensions.get::<UserSessionBeforeMiddleware>();

    match serde_json::to_string(&user_session) {
        Ok(user_session) => Ok(response(status::Ok, &user_session, application_json())),
        Err(err) => Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
    }
}

fn get_sessions(req: &mut Request) -> IronResult<Response> {
    let auth_required =  auth_required(req);
    let user_session = match req.extensions.get::<UserSessionBeforeMiddleware>() {
        Some(user_session) => Some(user_session.clone()),
        None => None,
    };

    if auth_required && user_session.is_none() {
        return Err(IronError::new(StringError(String::new()), status::Unauthorized));
    }

    let client = livy_client(req);

    let sessions = match client.get_sessions(None, None) {
        Ok(sessions) => sessions,
        Err(err) => {
            return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError))
        },
    };

    let mut sessions = match sessions.sessions {
        Some(sessions) => sessions,
        None => Vec::new(),
    };

    let (uid, is_admin) = match user_session {
        Some(user_session) => (user_session.uid.clone(), user_session.is_admin),
        None => (String::new(), false),
    };

    let sessions = sessions.iter_mut().filter(|ref session| {
        if !auth_required || is_admin {
            return true;
        }

        match session.proxy_user {
            Some(ref proxy_user) => proxy_user == uid.as_str(),
            None => false,
        }
    }).map(|session| {
        session.log = None;
        session
    }).collect::<Vec<_>>();

    let sessions = match serde_json::to_string(&sessions) {
        Ok(sessions) => sessions,
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
    };

    Ok(response(status::Ok, &sessions, application_json()))
}

fn kill_session(req: &mut Request) -> IronResult<Response> {
    let id = req.extensions.get::<Router>().unwrap()
        .find("id").unwrap().to_string();

    let id = match id.parse() {
        Ok(id) => id,
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
    };

    let client = livy_client(req);

    let user_session = match req.extensions.get::<UserSessionBeforeMiddleware>() {
        Some(user_session) => Some(user_session.clone()),
        None => None,
    };

    if !has_kill_session_authority(&client, id, auth_required(req), user_session.as_ref()) {
        return Err(IronError::new(StringError(String::new()), status::Unauthorized));
    }

    match client.kill_session(id) {
        Ok(_) => Ok(response(status::Ok, "{}", application_json())),
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
    }
}

fn has_kill_session_authority(client: &Client, id: i64, auth_required: bool, user_session: Option<&UserSession>) -> bool {
    if !auth_required {
        return true;
    }

    if user_session.is_none() {
        return false;
    }

    let user_session = user_session.unwrap();

    if user_session.is_admin {
        return true;
    }

    match client.get_session(id) {
        Ok(session) => {
            match session.proxy_user {
                Some(proxy_user) => {
                    proxy_user == user_session.uid
                },
                None => false
            }
        },
        Err(_) => false,
    }
}

fn text_html() -> Header<ContentType> {
    Header(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![(Attr::Charset, mime::Value::Utf8)])))
}

fn application_json() -> Header<ContentType> {
    Header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![(Attr::Charset, mime::Value::Utf8)])))
}

fn cache_control() -> Header<CacheControl> {
    Header(CacheControl(vec![
        CacheDirective::MustRevalidate,
        CacheDirective::NoCache,
        CacheDirective::NoStore,
        CacheDirective::Private,
    ]))
}

fn connection() -> Header<Connection> {
    Header(Connection::keep_alive())
}

fn response(status_code: Status, body: &str, content_type: Header<ContentType>) -> Response {
    Response::with((status_code, body, cache_control(), connection(), content_type))
}

fn redirect(status_code: Status, path: &str, uuid: Option<&str>) -> Response {
    Response::with((
        status_code,
        cache_control(),
        connection(),
        text_html(),
        Header(Location(path.to_owned())),
        set_cookie(uuid),
    ))
}

fn set_cookie(uuid: Option<&str>) -> Header<SetCookie> {
    match uuid {
        Some(uuid) => {
            let expires = time::now_utc() + Duration::days(7);
            Header(SetCookie(vec![format!("{}={}; expires={}; path=/", COOKIE_NAME, uuid, expires.rfc822())]))
        },
        None => {
            let expires = time::now_utc() - Duration::seconds(1);
            Header(SetCookie(vec![format!("{}=; expires={}; path=/", COOKIE_NAME, expires.rfc822())]))
        },
    }
}

fn livy_client(req: &mut Request) -> Client {
    let arc = req.get::<Read<Config>>().unwrap();
    let conf = arc.as_ref().livy_client.clone();

    Client::new(
        &conf.url,
        conf.gssnegotiate,
        conf.username
    )
}

/// User session
#[derive(Clone, Debug, Serialize)]
pub struct UserSession {
    pub uid: String,
    pub is_admin: bool,
}

pub struct UserSessionBeforeMiddleware;

impl Key for UserSessionBeforeMiddleware {
    type Value = UserSession;
}

impl BeforeMiddleware for UserSessionBeforeMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        if auth_required(req) {
            match get_uuid(&req.headers) {
                Some(uuid) => {
                    let arc = req.get::<State<UserSessions>>().unwrap();
                    let lock = arc.as_ref();
                    let user_sessions = lock.read().unwrap();
                    match user_sessions.map.get(&uuid) {
                        Some(user_session) => {
                            req.extensions.insert::<UserSessionBeforeMiddleware>(user_session.clone());
                        },
                        None => (),
                    }
                }
                None => (),
            }
        }
        Ok(())
    }
}

fn auth_required(req: &mut Request) -> bool {
    let arc = req.get::<Read<Config>>().unwrap();
    arc.as_ref().ldap.is_some()
}

pub fn get_uuid(headers: &Headers) -> Option<String> {
    for header in headers.iter() {
        if header.name() == "Cookie" {
            for kv in header.value_string().split(";") {
                let kv = kv.trim().split("=").collect::<Vec<&str>>();
                if kv.len() == 2 && kv[0] == COOKIE_NAME {
                    return Some(String::from(kv[1]));
                }
            }
        }
    }

    None
}

pub struct UserSessions {
    pub map: HashMap<String, UserSession>
}

impl UserSessions {
    pub fn new() -> UserSessions {
        UserSessions {
            map: HashMap::new(),
        }
    }
}

impl Key for UserSessions {
    type Value = Self;
}

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}
