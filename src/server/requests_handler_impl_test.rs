use std::sync::Arc;
use hyper::Uri;

use serde_json;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use http_client::HttpClient;
use testing_utils::testing_config;
use server::constants;
use server::requests_handler_impl::RequestsHandlerImpl;
use server::testing_server_wrapper;
use server::testing_server_wrapper::ServerWrapper;

fn start_server() -> ServerWrapper {
    let config = testing_config();
    let requests_handler = RequestsHandlerImpl::new(config);
    testing_server_wrapper::start_server(requests_handler.unwrap())
}

fn make_request(url: &str) -> JsonValue {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let response = http_client.make_request(Uri::from_str(url).unwrap());
    let mut tokio_core = tokio_core::reactor::Core::new().unwrap();
    let response = tokio_core.run(response).unwrap();
    serde_json::from_str(&response)
        .expect(&format!("Expected JSON response for query: {}, got: {}", url, response))
}

fn assert_status_ok(response: &JsonValue) {
    assert_status(response, constants::FIELD_STATUS_OK);
}

fn assert_status(response: &JsonValue, expected_status: &str) {
    match response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_STATUS) {
                Some(JsonValue::String(status)) => assert_eq!(status, expected_status),
                _ => panic!("Response must have status, but it didn't: {}", response)
            };
        },
        _ => panic!("Response expected to be json object, but was: {}", response)
    };
}

#[test]
fn test_register_client_cmd() {
    let server = start_server();

    let url = format!("http://{}{}?{}={}&{}={}&{}={}",
                      server.address(), &constants::CMD_REGISTER_USER,
                      &constants::ARG_USER_NAME, "name",
                      &constants::ARG_SOCIAL_NETWORK_TYPE, "vk",
                      &constants::ARG_SOCIAL_NETWORK_TOKEN, "token");
    let response = make_request(&url);
    assert_status_ok(&response);

    match &response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_USER_ID) {
                Some(JsonValue::String(uuid)) => {
                    Uuid::parse_str(&uuid).expect("Expecting valid uuid");
                },
                _ => panic!("Response must have uid, but it didn't: {}", response)
            }
        },
        _ => panic!("Response expected to be json object, but was: {}", response)
    };
}
//
//#[test]
//fn test_register_client_fails_when_no_social_network_type_provided() {
//    let server = start_server();
//
//    let url = format!("http://{}{}?{}={}&{}={}",
//                      server.address(), &constants::CMD_REGISTER_USER,
//                      &constants::ARG_USER_NAME, "name",
//                      &constants::ARG_SOCIAL_NETWORK_TYPE, "vk");
//    let response = make_request(&url);
//    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
//}
//
//#[test]
//fn test_register_client_fails_when_no_social_network_token_provided() {
//    let server = start_server();
//
//    let url = format!("http://{}{}?{}={}&{}={}",
//                      server.address(), &constants::CMD_REGISTER_USER,
//                      &constants::ARG_USER_NAME, "name",
//                      &constants::ARG_SOCIAL_NETWORK_TOKEN, "token");
//    let response = make_request(&url);
//    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
//}