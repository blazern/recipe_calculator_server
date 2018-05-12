use futures::future::Future;
use hyper;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use super::requests_handler::RequestsHandler;

struct EntryPoint<RH> where RH: RequestsHandler {
    requests_handler: Mutex<RefCell<RH>>,
}

impl<RH> EntryPoint<RH> where RH: RequestsHandler {
    fn new(requests_handler: RefCell<RH>) -> EntryPoint<RH> {
        EntryPoint{ requests_handler: Mutex::new(requests_handler) }
    }
}

impl<RH> Service for EntryPoint<RH> where RH: RequestsHandler {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let requests_handler = self.requests_handler.lock().unwrap();
        let str_future = requests_handler.borrow_mut().handle(req.path(), req.query());
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
    let requests_handler = RefCell::new(requests_handler);
    let entry_point = Arc::new(EntryPoint::new(requests_handler));
    // NOTE that passed closure is 'Fn' not 'FnOnce' for a reason - Hyper can call it many times,
    // assuming that each call creates a new service (but we always provide the same instance).
    let server = Http::new().bind(address, move || Ok(entry_point.clone())).unwrap();
    server.run_until(shutdown_signal).unwrap();
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;