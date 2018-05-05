use futures::future::Future;
use hyper;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use std::net::SocketAddr;
use std::sync::Arc;

use super::requests_handler::RequestsHandler;

struct EntryPoint {
    requests_handler: Arc<RequestsHandler>,
}

impl Service for EntryPoint {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let str_future = (*self.requests_handler).handle(req.uri());
        let future =
            str_future
            .map(|str_response| {
                Response::new()
                    .with_header(ContentLength(str_response.len() as u64))
                    .with_body(str_response)
            })
            .map_err(|_|panic!("Error not expected"));

        Box::new(future)
    }
}

// Starts server on the calling thread, blocking it.
pub fn start_server<F, RH>(address: &SocketAddr, shutdown_signal: F, requests_handler: RH) where
        F: Future<Item = (), Error = ()>,
        RH: RequestsHandler + 'static {
    let requests_handler = Arc::new(requests_handler);
    let entry_point = Arc::new(EntryPoint{ requests_handler });
    let server = Http::new().bind(address, move || Ok(entry_point.clone())).unwrap();
    server.run_until(shutdown_signal).unwrap();
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;