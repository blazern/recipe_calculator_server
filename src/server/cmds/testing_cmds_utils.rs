use hyper::Uri;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use serde_json;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::testing_util::testing_connection_for_server_user;
use crate::outside::http_client::HttpClient;
use crate::server::cmds::register_user::user_data_generators::create_gp_overrides;
use crate::server::constants;
use crate::server::requests_handler_impl::RequestsHandlerImpl;
use crate::server::testing_hostname;
use crate::server::testing_mock_server::FullRequest;
use crate::server::testing_mock_server::TestingMockServer;
use crate::server::testing_server_wrapper;
use crate::server::testing_server_wrapper::ServerWrapper;
use crate::testing_utils::{exhaust_future, testing_config};
use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;

#[macro_export]
macro_rules! start_server {
    () => {{
        use crate::server::cmds::start_pairing::start_pairing_cmd_handler::insert_pairing_code_gen_family_override;
        use crate::server::cmds::pairing_request::pairing_request_cmd_handler::insert_pairing_request_overrides;
        use crate::server::cmds::testing_cmds_utils::start_server_with_overrides;
        let fam = format!("{}{}", file!(), line!());
        let mut overrides = json!({});
        insert_pairing_code_gen_family_override(&mut overrides, fam.clone());
        insert_pairing_request_overrides(&mut overrides, fam);
        start_server_with_overrides(&overrides)
    }};
}

pub fn start_server_with_overrides(overrides: &JsonValue) -> ServerWrapper {
    let config = testing_config();
    // NOTE: address is acquired before handler requests creation for a reason -
    // address is a mutex lock, and multiple structs RequestsHandlerImpl can't live simultaneously
    let address = testing_hostname::get_hostname();
    let requests_handler = RequestsHandlerImpl::new_with_overrides(config, overrides);
    testing_server_wrapper::start_server(requests_handler.unwrap(), address)
}

pub fn start_mock_server<Responder>(
    responder: Responder,
    addr: MutexGuard<'static, String>,
) -> (ServerWrapper, Arc<Mutex<Vec<FullRequest>>>)
where
    Responder: Fn(&FullRequest) -> Option<String> + Send + Sync + 'static,
{
    let requests_handler = TestingMockServer::new(responder);
    let requests = requests_handler.received_requests.clone();
    let serv = testing_server_wrapper::start_server(requests_handler, addr);
    (serv, requests)
}

pub fn make_request(url: &str) -> JsonValue {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let response = http_client.req_get(Uri::from_str(url).unwrap());
    let response = exhaust_future(response).unwrap();
    serde_json::from_str(&response.body).unwrap_or_else(|_| {
        panic!(
            "Expected JSON response for query: {}, got: {:?}",
            url, response
        )
    })
}

pub fn assert_status_ok(response: &JsonValue) {
    assert_status(response, constants::FIELD_STATUS_OK);
}

pub fn assert_status(response: &JsonValue, expected_status: &str) {
    let status = response[constants::FIELD_NAME_STATUS]
        .as_str()
        .unwrap_or_else(|| panic!("Response must have status, but it didn't: {}", response));
    if status != expected_status {
        panic!("{} != {}, response: {}", expected_status, status, response)
    }
}

/// Cleaning up before tests
pub fn delete_app_user_with(uid: &Uuid) {
    use crate::db::core::util::delete_app_user;
    delete_app_user(&uid, &testing_connection_for_server_user().unwrap()).unwrap();
}

pub fn register_user(serv_address: &str, uuid: &Uuid, gp_uid: &str) -> JsonValue {
    register_named_user(serv_address, uuid, gp_uid, "name")
}

pub fn register_named_user(serv_address: &str, uuid: &Uuid, gp_uid: &str, name: &str) -> JsonValue {
    let gp_override = format!("{{ \"sub\": \"{}\" }}", gp_uid);
    let override_str = create_gp_overrides(&uuid, &gp_override);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        serv_address,
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        name,
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);
    response
}

pub fn set_user_fcm_token(
    serv_address: &str,
    client_token: &str,
    uid: &str,
    fcm_token: &str,
) -> JsonValue {
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        serv_address,
        &constants::CMD_UPDATE_FCM_TOKEN,
        &constants::ARG_USER_ID,
        percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_FCM_TOKEN,
        percent_encode(&fcm_token.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let response = make_request(&url);
    assert_status_ok(&response);
    response
}

pub fn start_pairing(server_addr: &str, client_token: &str, uid: &str) -> JsonValue {
    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server_addr,
        &constants::CMD_START_PAIRING,
        &constants::ARG_USER_ID,
        percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status_ok(&response);
    response
}

pub fn pairing_request_by_code(
    server_addr: &str,
    client_token: &str,
    uid: &str,
    code: &str,
) -> JsonValue {
    pairing_request_by_code_with_overrides(server_addr, client_token, uid, code, &json!({}))
}

pub fn pairing_request_by_code_with_overrides(
    server_addr: &str,
    client_token: &str,
    uid: &str,
    code: &str,
    overrides: &JsonValue,
) -> JsonValue {
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server_addr,
        &constants::CMD_PAIRING_REQUEST,
        &constants::ARG_USER_ID,
        percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_PAIRING_CODE,
        percent_encode(&code.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_OVERRIDES,
        percent_encode(&overrides.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status_ok(&response);
    response
}

pub fn pairing_request_by_uid(
    server_addr: &str,
    client_token: &str,
    uid: &str,
    partner_uid: &str,
) -> JsonValue {
    pairing_request_by_uid_with_overrides(server_addr, client_token, uid, partner_uid, &json!({}))
}

pub fn pairing_request_by_uid_with_overrides(
    server_addr: &str,
    client_token: &str,
    uid: &str,
    partner_uid: &str,
    overrides: &JsonValue,
) -> JsonValue {
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server_addr,
        &constants::CMD_PAIRING_REQUEST,
        &constants::ARG_USER_ID,
        percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&partner_uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_OVERRIDES,
        percent_encode(&overrides.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status_ok(&response);
    response
}
