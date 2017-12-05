use config::Config;
use frontend::html::index::INDEX;
use futures;
use futures::Stream;
use futures::future::Future;
use hyper;
use hyper::{Body, Chunk, Headers, Method, StatusCode};
use hyper::server::{Request, Response, Service};
use livy::v0_4_0::Client;
type LivyManagerResponse = Response<Box<Stream<Item=Chunk, Error=hyper::Error>>>;

/// Livy Manager
pub struct LivyManager {
    client: Client,
    conf: Config
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
            conf,
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
                get_sessions(&req)
            }
            _ => {
                not_found(&req)
            }
        };

        Box::new(futures::future::ok(res))
    }
}

fn index(req: &Request) -> LivyManagerResponse {
    let mut headers = Headers::new();
    headers.append_raw("Cache-Control", "private, no-store, no-cache, must-revalidate");
    headers.append_raw("Connection", "keep-alive");
    headers.append_raw("Content-Type", "text/html; charset=utf-8");

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(INDEX));

    Response::new().with_headers(headers).with_body(body)
}

fn get_sessions(req: &Request) -> LivyManagerResponse {
     Response::new().with_status(StatusCode::InternalServerError)
}

fn not_found(_: &Request) -> LivyManagerResponse {
    Response::new().with_status(StatusCode::NotFound)
}
