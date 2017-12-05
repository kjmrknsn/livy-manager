use config::Config;
use futures;
use futures::Stream;
use futures::future::Future;
use hyper;
use hyper::{Body, Chunk};
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
        let mut response = Response::new();
        let body: Box<Stream<Item=_, Error=_>> = Box::new(Body::from("Hello, Livy Manager"));
        response.set_body(body);
        Box::new(futures::future::ok(response))
    }
}
