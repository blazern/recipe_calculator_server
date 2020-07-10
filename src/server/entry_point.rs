use hyper::service::{make_service_fn, service_fn};
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;

use futures::future::select;
use futures::future::Future;
use futures::StreamExt;
use log::error;
use tokio::runtime::Runtime;

use std::cell::RefCell;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use super::requests_handler::RequestsHandler;
use std::collections::HashMap;

pub const MAX_BODY_SIZE: i32 = 1024 * 50;

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
    F: Future<Output = ()> + Unpin,
    RH: RequestsHandler + 'static,
{
    let requests_handler = RefCell::new(requests_handler);
    let entry_point = Arc::new(EntryPoint::new(requests_handler));

    let mut tokio_runtime = Runtime::new().expect("Tokio expected to be ok");

    // Server::bind will panic if it's executed not in Tokio runtime, so we pack it into a Future
    let serve_future = async {
        // Create a server bound on the provided address
        let serve_future = Server::bind(&address).serve(make_service_fn(move |_| {
            let entry_point = entry_point.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    handle_request_by_entry_point(req, entry_point.clone())
                }))
            }
        }));
        let server_with_shutdown_signal = select(shutdown_signal, serve_future);
        server_with_shutdown_signal.await;
    };

    tokio_runtime.block_on(serve_future);
}

async fn handle_request_by_entry_point<RH>(
    req: Request<Body>,
    entry_point: Arc<EntryPoint<RH>>,
) -> Result<Response<Body>, hyper::Error>
where
    RH: RequestsHandler + 'static,
{
    let uri = req.uri();

    let request = uri.path().to_string();
    let query = match uri.query() {
        Some(query) => query.to_string(),
        None => "".to_string(),
    };
    let headers = extract_headers(&req);
    let body = extract_body(&request, req).await;

    let body = match body {
        Ok(body) => body,
        Err(err_resp) => return Ok(err_resp),
    };

    let response_str = {
        // Lock is within narrowest scope
        let requests_handler = entry_point
            .requests_handler
            .lock()
            .expect("Broken mutex == broken app");
        let mut requests_handler = requests_handler.borrow_mut();
        requests_handler.handle(request, query, headers, body)
    };
    let response_str = response_str.await;
    Ok(Response::new(Body::from(response_str)))
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

async fn extract_body(request: &str, req: Request<Body>) -> Result<String, Response<Body>> {
    let mut bytes = Vec::new();

    let mut body_stream = req.into_body();
    while let Some(chunk) = body_stream.next().await {
        match chunk {
            Ok(chunk) => {
                bytes.append(&mut chunk.to_vec());
                if bytes.len() > MAX_BODY_SIZE as usize {
                    error!("Request body too large, uri: {}", request);
                    return Err(Response::builder()
                        .status(413) // Payload Too Large
                        .body(Body::from(""))
                        .expect("Expecting valid response"));
                }
            }
            Err(err) => {
                error!("Request body reading error, uri: {}, error: {:?}", request, err);
                return Err(Response::builder()
                    .status(500)
                    .body(Body::from(format!("{:?}", err)))
                    .expect("Expecting valid response"));
            }
        }
    }

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg(test)]
#[path = "./entry_point_test.rs"]
mod entry_point_test;
