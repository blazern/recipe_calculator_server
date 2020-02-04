use futures::future::ok;
use futures::Future;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use server::requests_handler::RequestsHandler;

pub struct TestingMockServer<Responder>
where
    Responder: Fn(&FullRequest) -> Option<String> + Send + Sync,
{
    pub received_requests: Arc<Mutex<Vec<FullRequest>>>,
    responses_generator: Responder,
}

#[derive(Debug)]
pub struct FullRequest {
    pub request: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl<Responder> TestingMockServer<Responder>
where
    Responder: Fn(&FullRequest) -> Option<String> + Send + Sync,
{
    pub fn new(responder: Responder) -> Self {
        TestingMockServer {
            received_requests: Arc::new(Mutex::new(Vec::new())),
            responses_generator: responder,
        }
    }
}

impl<Responder> RequestsHandler for TestingMockServer<Responder>
where
    Responder: Fn(&FullRequest) -> Option<String> + Send + Sync,
{
    fn handle(
        &mut self,
        request: String,
        query: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Box<dyn Future<Item = String, Error = ()> + Send> {
        let req = FullRequest {
            request,
            query,
            headers,
            body,
        };
        let response = match (self.responses_generator)(&req) {
            Some(response) => response,
            None => "".to_owned(),
        };
        self.received_requests.lock().unwrap().push(req);
        Box::new(ok(response))
    }
}
