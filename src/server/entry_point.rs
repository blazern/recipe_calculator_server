use futures::future::Future;

use futures;
use hyper;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use std::net::SocketAddr;

struct EntryPoint;

pub const RESPONSE: &'static str = "Hello, World!";

// Temporary implementation of server, copied from hyper's docs.
impl Service for EntryPoint {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'RESPONSE' body.
        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(RESPONSE.len() as u64))
                .with_body(RESPONSE)
        ))
    }
}

// Starts server on the calling thread, blocking it.
pub fn start_server<F>(address: &SocketAddr, shutdown_signal: F) where F: Future<Item = (), Error = ()> {
    let server = Http::new().bind(address, || Ok(EntryPoint)).unwrap();
    server.run_until(shutdown_signal).unwrap();
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;