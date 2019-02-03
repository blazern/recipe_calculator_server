use server::requests_handler::RequestsHandler;
use server::testing_server_wrapper;

use http_client;

struct Echo {
    string: String,
}

impl RequestsHandler for Echo {
    fn handle(&mut self, _request: &str, _query: Option<&str>) -> String {
        self.string.clone()
    }
}

#[test]
fn test_server_responses() {
    let expected_response = "Hello, world";
    let server = testing_server_wrapper::start_server(Echo{ string: expected_response.to_string() });

    let url = format!("http://{}", server.address());
    let response = http_client::make_blocking_request(&url).unwrap();

    assert_eq!(expected_response, response);
}
