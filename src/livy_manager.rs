use config::Config;
use frontend::html::index::INDEX;
use frontend::html::login::LOGIN;
use futures;
use futures::Stream;
use futures::future::Future;
use hyper;
use hyper::{Body, Chunk, Headers, Method, StatusCode};
use hyper::server::{Request, Response, Service};
use ldap;
use livy;
use percent_encoding;
use regex::Regex;
use serde_json;
use server::{UserSession, UserSessions};
use std::sync::{Arc, Mutex};
use time;
use time::Duration;
use uuid::Uuid;

type LivyManagerResponse = Response<Box<Stream<Item=Chunk, Error=hyper::Error>>>;

type LivyManagerResult = Result<LivyManagerResponse, String>;

const COOKIE_NAME: &'static str = "_lmsid";

/// Livy Manager
pub struct LivyManager {
    livy_client: livy::client::Client,
    ldap_client: Option<ldap::Client>,
    check_user_session: bool,
    user_sessions: Arc<Mutex<UserSessions>>,
}

impl LivyManager {
    /// Creates a new `LivyManger`.
    pub fn new(conf: Config, user_sessions: Arc<Mutex<UserSessions>>) -> LivyManager {
        let livy_client = livy::client::Client::new(
            &conf.livy_client.url,
            conf.livy_client.gssnegotiate.clone(),
            conf.livy_client.username.clone(),
        );

        let ldap_client = match conf.ldap {
            Some(ldap_conf) => Some(ldap::Client::new(ldap_conf)),
            None => None,
        };

        let check_user_session = match ldap_client {
            Some(_) => true,
            None => false,
        };

        LivyManager {
            livy_client,
            ldap_client,
            check_user_session,
            user_sessions,
        }
    }
}

impl Service for LivyManager {
    type Request = Request;
    type Response = LivyManagerResponse;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let user_session = get_user_session(&req, self.check_user_session, &self.user_sessions);

        let path = String::from(req.path());

        let res = match (req.method(), path.as_str()) {
            (&Method::Get, "/") => {
                index(self.check_user_session, user_session.as_ref())
            },
            (&Method::Get, "/login") => {
                login(self.check_user_session, user_session.as_ref())
            },
            (&Method::Post, "/login") => {
                return auth(req, self.check_user_session, user_session.as_ref(), self.ldap_client.as_ref(), &self.user_sessions)
            },
            (&Method::Get, "/logout") => {
                logout(self.check_user_session, user_session.as_ref(), &self.user_sessions)
            },
            (&Method::Get, "/api/user") => {
                get_user(self.check_user_session, user_session.as_ref())
            },
            (&Method::Get, "/api/sessions") => {
                get_sessions(&self.livy_client, self.check_user_session, user_session.as_ref())
            },
            (&Method::Delete, path) => {
                let re = Regex::new(r"^/api/sessions/(?P<id>\d+)$").unwrap();

                match re.captures(path) {
                    Some(caps) => {
                        kill_session(&self.livy_client, &caps["id"], self.check_user_session, user_session.as_ref())
                    },
                    None => not_found(),
                }
            },
            _ => {
                not_found()
            }
        };

        let res = match res {
            Ok(res) => res,
            Err(err) => {
                eprintln!("error occurred: {}", err);
                Response::new().with_status(StatusCode::InternalServerError)
            },
        };

        Box::new(futures::future::ok(res))
    }
}

fn index(check_user_session: bool, user_session: Option<&UserSession>) -> LivyManagerResult {
    if check_user_session && user_session.is_none() {
        return Ok(redirect("/login", true));
    }

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(INDEX));

    Ok(Response::new().with_headers(html_headers()).with_body(body))
}

fn login(check_user_session: bool, user_session: Option<&UserSession>) -> LivyManagerResult {
    if !check_user_session || user_session.is_some() {
        return Ok(redirect("/", false));
    }

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(LOGIN));

    Ok(Response::new().with_headers(html_headers()).with_body(body))
}

fn auth(req: Request, check_user_session: bool, user_session: Option<&UserSession>, ldap_client: Option<&ldap::Client>, user_sessions: &Arc<Mutex<UserSessions>>) -> Box<Future<Item=LivyManagerResponse, Error=hyper::Error>> {
    if !check_user_session || user_session.is_some() {
        return Box::new(futures::future::ok(redirect("/", false)));
    }

    let ldap_client = ldap_client.cloned();
    let user_sessions = Arc::clone(user_sessions);

    Box::new(req.body().concat2().and_then(move |body| {
        let vec = body.iter().cloned().collect();
        let body = String::from_utf8(vec).unwrap();
        let (uid, password) = extract_uid_password(body.as_str());
        match ldap_client.unwrap().auth(uid.as_str(), password.as_str()) {
            Ok(user_session) => {
                let uuid = Uuid::new_v4().to_string();

                let mut user_sessions = user_sessions.lock().unwrap();
                if user_sessions.contains_key(uuid.as_str()) {
                    return futures::future::ok(redirect_see_other("/login?result=failed", None))
                }

                user_sessions.insert(uuid.clone(), user_session);
                futures::future::ok(redirect_see_other("/", Some(uuid.as_str())))
            },
            Err(_) => {
                futures::future::ok(redirect_see_other("/login?result=failed", None))
            },
        }
    }))
}

fn extract_uid_password(body: &str) -> (String, String) {
    let mut uid = String::new();
    let mut password = String::new();

    for kv in body.trim().split("&") {
        let kv = kv.split("=").collect::<Vec<&str>>();
        if kv.len() == 2 {
            match kv[0] {
                "uid" | "password" => {
                    if let Ok(v) = percent_encoding::percent_decode(kv[1].as_bytes()).decode_utf8() {
                        if kv[0] == "uid" {
                            uid = v.to_string();
                        } else {
                            password = v.to_string();
                        }
                    }
                },
                _ => (),
            }
        }
    }

    (uid, password)
}

fn logout(check_user_session: bool, user_session: Option<&UserSession>, user_sessions: &Arc<Mutex<UserSessions>>) -> LivyManagerResult {
    if !check_user_session {
        return Ok(redirect("/", true));
    }

    if user_session.is_none() {
        return Ok(redirect("/login", true));
    }

    let mut user_sessions = user_sessions.lock().unwrap();
    user_sessions.remove(user_session.unwrap().uid.as_str());
    Ok(redirect("/login", true))
}

fn get_user(check_user_session: bool, user_session: Option<&UserSession>) -> LivyManagerResult {
    if check_user_session && user_session.is_none() {
        return Ok(Response::new().with_status(StatusCode::Unauthorized));
    }

    match serde_json::to_string(&user_session) {
        Ok(user_session) => {
            let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(user_session));

            Ok(Response::new().with_headers(json_headers()).with_body(body))
        },
        Err(err) => Err(format!("{}", err)),
    }
}

fn get_sessions(client: &livy::client::Client, check_user_session: bool, user_session: Option<&UserSession>) -> LivyManagerResult {
    if check_user_session && user_session.is_none() {
        return Ok(Response::new().with_status(StatusCode::Unauthorized));
    }

    let sessions = match client.get_sessions(None, None) {
        Ok(result) => result,
        Err(err) => return Err(format!("{}", err)),
    };

    let mut sessions = match sessions.sessions {
        Some(sessions) => sessions,
        None => Vec::new(),
    };

    let (uid, is_admin) = match user_session {
        Some(user_session) => (user_session.uid.as_str(), user_session.is_admin),
        None => ("", false),
    };

    let sessions = sessions.iter_mut().filter(|ref session| {
        if !check_user_session || is_admin {
            return true;
        }
        match session.proxy_user {
            Some(ref proxy_user) => proxy_user == uid,
            None => false,
        }
    }).map(|session| {
        session.log = None;
        session
    }).collect::<Vec<_>>();

    let sessions = match serde_json::to_string(&sessions) {
        Ok(sessions) => sessions,
        Err(err) => return Err(format!("{}", err)),
    };

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(sessions));

    Ok(Response::new().with_headers(json_headers()).with_body(body))
}

fn kill_session(client: &livy::client::Client, id: &str, check_user_session: bool, user_session: Option<&UserSession>) -> LivyManagerResult {
    let id = match id.parse() {
        Ok(id) => id,
        Err(err) => return Err(format!("{}", err)),
    };

    if check_user_session {
        match user_session {
            Some(user_session) => {
                if !user_session.is_admin {
                    match client.get_session(id) {
                        Ok(session) => {
                            match session.proxy_user {
                                Some(proxy_user) => {
                                    if proxy_user != user_session.uid {
                                        return Ok(Response::new().with_status(StatusCode::Unauthorized));
                                    }
                                },
                                None => return Ok(Response::new().with_status(StatusCode::Unauthorized)),
                            }
                        },
                        Err(_) => return Ok(Response::new().with_status(StatusCode::Unauthorized)),
                    }
                }
            },
            None => return Ok(Response::new().with_status(StatusCode::Unauthorized)),
        }
    }

    match client.kill_session(id) {
        Ok(_) => {
            let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from("{}"));
            Ok(Response::new().with_headers(json_headers()).with_body(body))
        },
        Err(err) => Err(format!("{}", err)),
    }
}

fn not_found() -> LivyManagerResult {
    Ok(Response::new().with_status(StatusCode::NotFound))
}

fn headers() -> Headers {
    let mut headers = Headers::new();

    headers.append_raw("Cache-Control", "private, no-store, no-cache, must-revalidate");
    headers.append_raw("Connection", "keep-alive");

    headers
}

fn html_headers() -> Headers {
    let mut headers = headers();

    headers.append_raw("Content-Type", "text/html; charset=utf-8");

    headers
}

fn json_headers() -> Headers {
    let mut headers = headers();

    headers.append_raw("Content-Type", "application/json; charset=utf-8");

    headers
}

fn redirect(location: &str, delete_cookie: bool) -> LivyManagerResponse {
    let mut headers = headers();

    headers.append_raw("Content-Type", "text/html; charset=utf-8");
    headers.append_raw("Location", location);

    if delete_cookie {
        let expires = time::now_utc() - Duration::seconds(1);
        headers.append_raw("Set-Cookie", format!("{}=; expires={}; path=/", COOKIE_NAME, expires.rfc822()));

    }

    Response::new().with_status(StatusCode::TemporaryRedirect).with_headers(headers)
}

fn redirect_see_other(location: &str, uuid: Option<&str>) -> LivyManagerResponse {
    let mut headers = headers();

    headers.append_raw("Content-Type", "text/html; charset=utf-8");
    headers.append_raw("Location", location);

    if let Some(uuid) = uuid {
        let expires = time::now_utc() + Duration::days(7);
        headers.append_raw("Set-Cookie", format!("{}={}; expires={}; path=/", COOKIE_NAME, uuid, expires.rfc822()));

    }
    Response::new().with_status(StatusCode::SeeOther).with_headers(headers)
}
fn get_user_session(req: &Request, check_user_session: bool, user_sessions: &Arc<Mutex<UserSessions>>) -> Option<UserSession> {
    if !check_user_session {
        return None;
    }

    let mut sid = String::new();

    for header in req.headers().iter() {
        if header.name() == "Cookie" {
            for kv in header.value_string().split(";") {
                let kv = kv.trim().split("=").collect::<Vec<&str>>();
                if kv.len() == 2 && kv[0] == COOKIE_NAME {
                    sid = String::from(kv[1]);
                }
            }
        }
    }

    match sid.as_str() {
        "" => None,
        sid => {
            let user_sessions = user_sessions.lock().unwrap();
            user_sessions.get(sid).cloned()
        }
    }
}
