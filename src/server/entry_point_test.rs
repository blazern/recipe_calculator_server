use futures::future;
use futures::future::Future;
use reqwest::Client;
use std::io::Read;

use server::requests_handler::RequestsHandler;
use server::testing_hostname;
use server::testing_server_wrapper;

struct Echo {
    string: String,
}

impl RequestsHandler for Echo {
    fn handle(&mut self, _request: &str, _query: Option<&str>) -> Box<Future<Item=String, Error=()>> {
        Box::new(future::ok(self.string.clone()))
    }
}

#[test]
fn test_server_responses() {
    let address = testing_hostname::get_hostname();

    let expected_response = "Hello, world";
    let _server = testing_server_wrapper::start_server(&address,Echo{ string: expected_response.to_string() });
    let client = Client::new().unwrap();

    let url = format!("http://{}", &address);
    let mut response = client.get(&url).unwrap().send().unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();

    assert_eq!(expected_response, response_str);
}
