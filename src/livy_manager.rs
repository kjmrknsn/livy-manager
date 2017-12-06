use config::Config;
use frontend::html::index::INDEX;
use futures;
use futures::Stream;
use futures::future::Future;
use hyper;
use hyper::{Body, Chunk, Headers, Method, StatusCode};
use hyper::server::{Request, Response, Service};
use livy::client::Client;
use regex::Regex;
use serde_json;

type LivyManagerResponse = Response<Box<Stream<Item=Chunk, Error=hyper::Error>>>;

type LivyManagerResult = Result<LivyManagerResponse, String>;

/// Livy Manager
pub struct LivyManager {
    client: Client,
}

impl LivyManager {
    /// Creates a new `LivyManger`.
    pub fn new(conf: Config) -> LivyManager {
        let client = Client::new(
            &conf.livy_client.url,
            conf.livy_client.gssnegotiate.clone(),
            conf.livy_client.username.clone(),
        );

        LivyManager {
            client,
        }
    }
}

impl Service for LivyManager {
    type Request = Request;
    type Response = LivyManagerResponse;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let res = match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                index(&req)
            },
            (&Method::Get, "/api/sessions") => {
                get_sessions(&req, &self.client)
            },
            (&Method::Delete, path) => {
                let re = Regex::new(r"^/api/sessions/(?P<id>\d+)$").unwrap();

                match re.captures(path) {
                    Some(caps) => {
                        kill_session(&req, &self.client, &caps["id"])
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

fn index(_: &Request) -> LivyManagerResult {
    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(INDEX));

    Ok(Response::new().with_headers(html_headers()).with_body(body))
}

fn get_sessions(_: &Request, client: &Client) -> LivyManagerResult {
    let sessions = match client.get_sessions(None, None) {
        Ok(result) => result,
        Err(err) => return Err(format!("{}", err)),
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
        Err(err) => return Err(format!("{}", err)),
    };

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(sessions));

    Ok(Response::new().with_headers(json_headers()).with_body(body))
}

fn kill_session(_: &Request, client: &Client, id: &str) -> LivyManagerResult {
    let id = match id.parse() {
        Ok(id) => id,
        Err(err) => return Err(format!("{}", err)),
    };

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
