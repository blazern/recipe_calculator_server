use reqwest::Client;
use serde_json;
use serde_json::Value as JsonValue;
use std::io::Read;
use uuid::Uuid;

use testing_config;
use server::constants;
use server::requests_handler_impl::RequestsHandlerImpl;
use server::testing_hostname;
use server::testing_server_wrapper;

#[test]
fn test_register_client_cmd() {
    let address = testing_hostname::get_hostname();
    let config = testing_config::get();
    let requests_handler = RequestsHandlerImpl::new(config);

    let _server = testing_server_wrapper::start_server(&address, requests_handler);;
    let client = Client::new().unwrap();

    let url = format!("http://{}{}", &address, &constants::CMD_REGISTER_DEVICE);
    let mut response = client.get(&url).unwrap().send().unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();

    let device_id: Uuid;
    let response_json: JsonValue = serde_json::from_str(&response_str).expect("Expecting JSON");
    match response_json {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_STATUS) {
                Some(JsonValue::String(status)) => assert_eq!(status, constants::FIELD_STATUS_OK),
                _ => panic!("Response must have status, but it didn't: {}", response_str)
            };
            match fields.get(constants::FIELD_NAME_DEVICE_ID) {
                Some(JsonValue::String(uuid)) => {
                    device_id = Uuid::parse_str(uuid).expect("Expecting valid uuid");
                },
                _ => panic!("Response must have device ID, but it didn't: {}", response_str)
            };
        },
        _ => panic!("Response expected to be json object, but was: {}", response_str)
    }


    let url = format!("http://{}{}?{}={}",
                      &address, &constants::CMD_IS_DEVICE_REGISTERED,
                      &constants::ARG_DEVICE_ID, device_id.to_string());
    let mut response = client.get(&url).unwrap().send().unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();
    let response_json: JsonValue = serde_json::from_str(&response_str).expect("Expecting JSON");
    match response_json {
        JsonValue::Object(fields) => {
            match fields.get(constants::FIELD_NAME_REGISTERED) {
                Some(JsonValue::Bool(is_registered)) => assert!(is_registered),
                _ => panic!("Expected response with registration status, got: {}", response_str)
            }
        },
        _ => panic!("Response expected to be json object, but was: {}", response_str)
    }
}