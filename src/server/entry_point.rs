use hyper::rt::Future;
use hyper::service::Service;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;

use futures::future;
use futures::IntoFuture;
use tokio_core;

use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use crate::error::Never;

use super::requests_handler::RequestsHandler;
use std::collections::HashMap;

struct EntryPoint<RH>
where
    RH: RequestsHandler,
{
    requests_handler: Arc<Mutex<RefCell<RH>>>,
}

impl<RH> EntryPoint<RH>
where
    RH: RequestsHandler,
{
    fn new(requests_handler: RefCell<RH>) -> EntryPoint<RH> {
        EntryPoint {
            requests_handler: Arc::new(Mutex::new(requests_handler)),
        }
    }
}

// Starts server on the calling thread, blocking it.
pub fn start_server<F, RH>(address: &SocketAddr, shutdown_signal: F, requests_handler: RH)
where
    F: Future<Item = (), Error = ()>,
    RH: RequestsHandler + 'static,
{
    let requests_handler = RefCell::new(requests_handler);
    let entry_point = Arc::new(EntryPoint::new(requests_handler));

    let mut tokio_core = tokio_core::reactor::Core::new().expect("Tokio expected to be ok");

    let new_service = move || MyHyperService {
        entry_point: entry_point.clone(),
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
        Ok(_) => {}
        Err(_) => unreachable!("Server finished with unexpected error"),
    }
}

struct MyHyperService<RH>
where
    RH: RequestsHandler + 'static,
{
    entry_point: Arc<EntryPoint<RH>>,
}
impl<RH> Service for MyHyperService<RH>
where
    RH: RequestsHandler + 'static,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Never; // No errors
    type Future = Box<dyn Future<Item = Response<Body>, Error = Never> + Send>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let uri = req.uri();

        let request = uri.path().to_string();
        let query = match uri.query() {
            Some(query) => query.to_string(),
            None => "".to_string(),
        };
        let headers = extract_headers(&req);
        let body = extract_body(req);

        let requests_handler = self.entry_point.requests_handler.clone();

        let response = body.and_then(move |body| {
            let requests_handler = requests_handler.lock().expect("Broken mutex == broken app");

            let mut requests_handler = requests_handler.borrow_mut();
            requests_handler
                .handle(request, query, headers, body)
                .map(|response_str| Response::new(Body::from(response_str)))
                .map_err(|err| panic!("No errors were expected, got: {:?}", err))
        });

        Box::new(response)
    }
}
impl<RH> IntoFuture for MyHyperService<RH>
where
    RH: RequestsHandler,
{
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Never;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

fn extract_headers(req: &Request<Body>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for header in req.headers() {
        let key = header.0.as_str().to_owned();
        let val = match header.1.to_str() {
            Ok(val) => val.to_owned(),
            Err(_) => continue,
        };
        headers.insert(key, val);
    }
    headers
}

// Body extraction currently only supported for tests
#[cfg(not(test))]
fn extract_body(_req: Request<Body>) -> impl Future<Item = String, Error = Never> + Send {
    futures::future::ok("".to_owned())
}

#[cfg(test)]
fn extract_body(req: Request<Body>) -> impl Future<Item = String, Error = Never> + Send {
    // NOTE: this implementation is terrible and should be used
    // for tests only (for example, it doesn't check for body length,
    // and malicious client might break our server by providing a huge body).
    use futures::future::ok;
    use futures::Stream;

    req.into_body()
        .concat2()
        .and_then(|c| ok(String::from_utf8(c.to_vec()).unwrap()))
        .map_err(|err| panic!("Err: {:?}", err))
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;
