use cmd_args::CmdArgs;
use config::Config;
use frontend::html::index::INDEX;
use frontend::html::login::LOGIN;
use iron::headers::{CacheControl, CacheDirective, Connection, ContentType, Location};
use iron::mime;
use iron::mime::{Attr, Mime, TopLevel, SubLevel};
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use iron::status::Status;
use iron::Timeouts;
use ldap;
use livy::client::Client;
use params;
use params::Params;
use persistent::Read;
use router::Router;
use serde_json;
use std::error::Error;
use std::fmt::{self, Debug};

pub fn run() {
    let args = CmdArgs::new();

    if args.print_version {
        println!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let conf = Config::from(&args.conf_path);

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/login", login, "login");
    router.post("/login", auth, "auth");
    router.get("/api/sessions", get_sessions, "get_sessions");
    router.delete("/api/sessions/:id", kill_session, "kill_session");

    eprintln!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
    eprintln!("Listening on {}.", conf.http.addr);

    let mut chain = Chain::new(router);
    chain.link(Read::<Config>::both(conf.clone()));

    let iron = Iron {
        handler: chain,
        timeouts: Timeouts::default(),
        threads: conf.http.num_threads,
    };
    iron.http(conf.http.addr).unwrap();
}

fn index(_: &mut Request) -> IronResult<Response> {
    Ok(response(status::Ok, INDEX, text_html()))
}

fn login(_: &mut Request) -> IronResult<Response> {
    Ok(response(status::Ok, LOGIN, text_html()))
}

fn auth(req: &mut Request) -> IronResult<Response> {
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
                    Ok(response(status::Ok, "{}", application_json()))
                },
                Err(_) => Ok(redirect(status::SeeOther, "/login?result=failed"))
            }
        },
        _ => Err(IronError::new(StringError("invalid parameters".to_string()), status::BadRequest)),
    }
}

fn get_sessions(req: &mut Request) -> IronResult<Response> {
    let client = livy_client(req);

    let sessions = match client.get_sessions(None, None) {
        Ok(sessions) => sessions,
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
    };

    let mut sessions = match sessions.sessions {
        Some(sessions) => sessions,
        None => Vec::new(),
    };

    let sessions = sessions.iter_mut().map(|session| {
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

    match client.kill_session(id) {
        Ok(_) => Ok(response(status::Ok, "{}", application_json())),
        Err(err) => return Err(IronError::new(StringError(format!("{}", err)), status::InternalServerError)),
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

fn redirect(status_code: Status, path: &str) -> Response {
    Response::with((status_code, cache_control(), connection(), text_html(), Header(Location(path.to_owned()))))
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
