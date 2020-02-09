use futures::future::ready;
use futures::Future;
use hyper::Uri;
use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use crate::outside::http_client::{HttpClient, RequestMethod, Response};
use crate::server::entry_point::MAX_BODY_SIZE;
use crate::server::requests_handler::RequestsHandler;
use crate::server::testing_hostname;
use crate::server::testing_server_wrapper;
use crate::testing_utils::exhaust_future;

struct Echo {
    string: String,
}
impl RequestsHandler for Echo {
    fn handle(
        &mut self,
        _request: String,
        _query: String,
        _headers: HashMap<String, String>,
        _body: String,
    ) -> Pin<Box<dyn Future<Output = String> + Send>> {
        Box::pin(ready(self.string.clone()))
    }
}

fn make_request(url: &str) -> Response {
    make_request_with_body(url, "".to_owned())
}

fn make_request_with_body(url: &str, body: String) -> Response {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let response = http_client.req(
        Uri::from_str(url).unwrap(),
        RequestMethod::Post,
        HashMap::new(),
        Some(body),
    );
    exhaust_future(response).unwrap()
}

#[test]
fn test_server_responses() {
    let expected_response = "Hello, world";
    let address = testing_hostname::get_hostname();
    let server = testing_server_wrapper::start_server(
        Echo {
            string: expected_response.to_string(),
        },
        address,
    );

    let url = format!("http://{}", server.address());
    let response = make_request(&url);

    assert_eq!(expected_response, response.body);
}

#[test]
fn test_too_large_body() {
    let address = testing_hostname::get_hostname();
    let expected_response = "";
    let server = testing_server_wrapper::start_server(
        Echo {
            string: expected_response.to_string(),
        },
        address,
    );

    let large_body: Vec<u8> = vec![1; (MAX_BODY_SIZE + 1) as usize];

    let url = format!("http://{}", server.address());
    let response = make_request_with_body(&url, String::from_utf8(large_body).unwrap());

    let expected_status_code = 413;
    assert_eq!(expected_response, response.body);
    assert_eq!(expected_status_code, response.status_code);
}
