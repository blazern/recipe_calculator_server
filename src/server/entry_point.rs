use hyper::Body;
use hyper::Response;
use hyper::Server;
use hyper::Request;
use hyper::service::Service;
use hyper::rt::Future;

use tokio_core;
use futures::IntoFuture;
use futures::future;

use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use error::Never;

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

    let mut tokio_core = tokio_core::reactor::Core::new().expect("Tokio expected to be ok");

    let new_service = move || {
        MyHyperService{ entry_point: entry_point.clone() }
    };

    let shutdown_signal = shutdown_signal.map_err(|err| {
        panic!("No errors expected from shutdown signal, got: {:?}", err);
    });
    let server = Server::bind(&address)
        .serve(new_service)
        .map_err(|err| panic!("No error was expected, but got: {:?}", err));

    let server_with_shutdown_signal = shutdown_signal.select(server);
    let server_result = tokio_core.run(server_with_shutdown_signal);
    match server_result {
        Ok(_) => {},
        Err(_) => {
            panic!("Server finished with unexpected error")
        }
    }
}

struct MyHyperService<RH> where RH: RequestsHandler  {
    entry_point: Arc<EntryPoint<RH>>
}
impl<RH> Service for MyHyperService<RH> where RH: RequestsHandler {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Never; // No errors
    type Future = Box<dyn Future<Item=Response<Body>, Error=Never> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let uri = req.uri();

        let requests_handler = self.entry_point.requests_handler.lock().expect("Broken mutex == broken app");
        let request = uri.path().to_string();
        let query = match uri.query() {
            Some(query) => query.to_string(),
            None => "".to_string(),
        };
        let response = requests_handler.borrow_mut().handle(request, query)
            .map(|response_str| Response::new(Body::from(response_str)))
            .map_err(|err| panic!("No errors were expected, got: {:?}", err));
        Box::new(response)
    }
}
impl<RH> IntoFuture for MyHyperService<RH> where RH: RequestsHandler {
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Never;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;
