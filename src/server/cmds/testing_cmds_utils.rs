use hyper::Uri;
use std::sync::Arc;

use serde_json;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use db::core::testing_util::testing_connection_for_server_user;
use outside::http_client::HttpClient;
use server::cmds::register_user::user_data_generators::create_gp_overrides;
use server::constants;
use server::requests_handler_impl::RequestsHandlerImpl;
use server::testing_server_wrapper;
use server::testing_server_wrapper::ServerWrapper;
use testing_utils::testing_config;

#[macro_export]
macro_rules! start_server {
    () => {{
        use server::cmds::start_pairing::start_pairing_cmd_handler::insert_pairing_code_gen_family_override;
        use server::cmds::testing_cmds_utils::start_server_with_overrides;
        let fam = format!("{}{}", file!(), line!());
        let mut overrides = json!({});
        insert_pairing_code_gen_family_override(&mut overrides, fam);
        start_server_with_overrides(&overrides)
    }};
}

pub fn start_server_with_overrides(overrides: &JsonValue) -> ServerWrapper {
    let config = testing_config();
    let requests_handler = RequestsHandlerImpl::new_with_overrides(config, overrides);
    testing_server_wrapper::start_server(requests_handler.unwrap())
}

pub fn make_request(url: &str) -> JsonValue {
    let http_client = Arc::new(HttpClient::new().unwrap());
    let response = http_client.req_get(Uri::from_str(url).unwrap());
    let mut tokio_core = tokio_core::reactor::Core::new().unwrap();
    let response = tokio_core.run(response).unwrap();
    serde_json::from_str(&response).unwrap_or_else(|_| {
        panic!(
            "Expected JSON response for query: {}, got: {}",
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
    use db::core::util::delete_app_user;
    delete_app_user(&uid, &testing_connection_for_server_user().unwrap()).unwrap();
}

pub fn register_user(serv_address: &str, uuid: &Uuid, gp_uid: &str) -> JsonValue {
    let gp_override = format!("{{ \"sub\": \"{}\" }}", gp_uid);
    let override_str = create_gp_overrides(&uuid, &gp_override);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        serv_address,
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
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
