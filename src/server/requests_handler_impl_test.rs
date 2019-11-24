use serde_json;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use http_client;
use testing_config;
use server::constants;
use server::requests_handler_impl::RequestsHandlerImpl;
use server::testing_server_wrapper;
use server::testing_server_wrapper::ServerWrapper;

fn start_server() -> ServerWrapper {
    let config = testing_config::get();
    let requests_handler = RequestsHandlerImpl::new(config);
    testing_server_wrapper::start_server(requests_handler.unwrap())
}

fn make_request(url: &str) -> JsonValue {
    let response = http_client::make_blocking_request(url).unwrap();
    serde_json::from_str(&response)
        .expect(&format!("Expected JSON response for query: {}, got: {}", url, response))
}

fn assert_status_ok(response: &JsonValue) {
    match response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_STATUS) {
                Some(JsonValue::String(status)) => assert_eq!(status, constants::FIELD_STATUS_OK),
                _ => panic!("Response must have status, but it didn't: {}", response)
            };
        },
        _ => panic!("Response expected to be json object, but was: {}", response)
    };
}

#[test]
fn random_client_doesnt_exists() {
    let server = start_server();
    // NOTE: there's close to 0 possibility that the generated UUID exists in DB
    let random_uuid = Uuid::new_v4();
    let url = format!("http://{}{}?{}={}",
                      server.address(), &constants::CMD_IS_DEVICE_REGISTERED,
                      &constants::ARG_DEVICE_ID, random_uuid.to_string());

    let response = make_request(&url);
    assert_status_ok(&response);

    match &response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_REGISTERED) {
                Some(JsonValue::Bool(is_registered)) => assert!(!is_registered),
                _ => panic!("Expected response with registration status, got: {}", &response)
            };
        },
        _ => panic!("Response expected to be json object, but was: {}", &response)
    };
}

#[test]
fn test_register_client_cmd() {
    let server = start_server();

    let url = format!("http://{}{}", server.address(), &constants::CMD_REGISTER_DEVICE);
    let response = make_request(&url);
    assert_status_ok(&response);

    let device_id: Uuid;
    match &response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_DEVICE_ID) {
                Some(JsonValue::String(uuid)) => {
                    device_id = Uuid::parse_str(uuid).expect("Expecting valid uuid");
                },
                _ => panic!("Response must have device ID, but it didn't: {}", response)
            }
        },
        _ => panic!("Response expected to be json object, but was: {}", response)
    };


    let url = format!("http://{}{}?{}={}",
                      server.address(), &constants::CMD_IS_DEVICE_REGISTERED,
                      &constants::ARG_DEVICE_ID, device_id.to_string());
    let response = make_request(&url);
    match &response {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_REGISTERED) {
                Some(JsonValue::Bool(is_registered)) => assert!(is_registered),
                _ => panic!("Expected response with registration status, got: {}", response)
            };
        },
        _ => panic!("Response expected to be json object, but was: {}", response)
    };
}
