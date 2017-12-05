use config::Config;
use frontend::html::index::INDEX;
use futures;
use futures::Stream;
use futures::future::Future;
use hyper;
use hyper::{Body, Chunk, Headers, Method, StatusCode};
use hyper::server::{Request, Response, Service};

/// Livy Manager
pub struct LivyManager {
    conf: Config
}

impl LivyManager {
    /// Creates a new `LivyManger`.
    pub fn new(conf: Config) -> LivyManager {
        LivyManager {
            conf,
        }
    }
}

impl Service for LivyManager {
    type Request = Request;
    type Response = Response<Box<Stream<Item=Chunk, Error=Self::Error>>>;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let res = match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                index(&req)
            },
            _ => {
                not_found(&req)
            }
        };

        Box::new(futures::future::ok(res))
    }
}

fn index(req: &Request) -> Response<Box<Stream<Item=Chunk, Error=hyper::Error>>> {
    let mut headers = Headers::new();
    headers.append_raw("Cache-Control", "private, no-store, no-cache, must-revalidate");
    headers.append_raw("Connection", "keep-alive");
    headers.append_raw("Content-Type", "text/html; charset=utf-8");

    let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from(INDEX));

    Response::new().with_headers(headers).with_body(body)
}

fn not_found(_: &Request) -> Response<Box<Stream<Item=Chunk, Error=hyper::Error>>> {
    let mut res = Response::new();

    res.set_status(StatusCode::NotFound);

    res
}
