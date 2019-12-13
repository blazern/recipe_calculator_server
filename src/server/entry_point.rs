use hyper::Body;
use hyper::Response;
use hyper::Server;
use hyper::service::service_fn_ok;
use hyper::rt::Future;

use tokio_core;

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

// Starts server on the calling thread, blocking it.
pub fn start_server<F, RH>(address: &SocketAddr, shutdown_signal: F, requests_handler: RH) where
        F: Future<Item = (), Error = ()>,
        RH: RequestsHandler + 'static {
    let requests_handler = RefCell::new(requests_handler);
    let entry_point = Arc::new(EntryPoint::new(requests_handler));

    let mut tokio_core = tokio_core::reactor::Core::new().unwrap(); // TODO remove unwrap

    let new_service = move || {
        let entry_point = entry_point.clone();
        service_fn_ok(move |req| {
            let uri = req.uri();

            let requests_handler = entry_point.requests_handler.lock().expect("Broken mutex == broken app");
            let request = uri.path().to_string();
            let query = match uri.query() {
                Some(query) => query.to_string(),
                None => "".to_string(),
            };
            let response = requests_handler.borrow_mut().handle(request, query);
            let str_response = response.wait().expect("Request handling is not expected to ever send an error");
            Response::new(Body::from(str_response))
        })
    };

    let shutdown_signal = shutdown_signal.map_err(|err| {
        panic!("No errors expected from shutdown signal, got: {:?}", err);
    });
    let server = Server::bind(&address)
        .serve(new_service)
        .map_err(|e| panic!("No error was expected, but got: {}", e));

    let server_with_shutdown_signal = shutdown_signal.select(server);
    let server_result = tokio_core.run(server_with_shutdown_signal);
    match server_result {
        Ok(_) => {},
        Err(_) => {
            panic!("Server finished with unexpected error")
        }
    }
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;
