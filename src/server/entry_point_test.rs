use futures::future::ok;
use futures::Future;
use hyper::Uri;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::server::requests_handler::RequestsHandler;
use crate::server::testing_hostname;
use crate::server::testing_server_wrapper;

use crate::outside::http_client::HttpClient;

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
    ) -> Box<dyn Future<Item = String, Error = ()> + Send> {
        Box::new(ok(self.string.clone()))
    }
}

fn make_request(url: &str) -> String {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let response = http_client.req_get(Uri::from_str(url).unwrap());
    let mut tokio_core = tokio_core::reactor::Core::new().unwrap();
    tokio_core.run(response).unwrap()
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

    assert_eq!(expected_response, response);
}
