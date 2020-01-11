use hyper::Uri;
use std::sync::Arc;

use serde_json;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::gp_user;
use db::core::testing_util::testing_connection_for_server_user;
use db::core::vk_user;
use http_client::HttpClient;
use server::cmds::register_user::user_data_generators::create_gp_overrides;
use server::cmds::register_user::user_data_generators::create_uuid_only_overrides;
use server::cmds::register_user::user_data_generators::create_vk_overrides;
use server::constants;
use server::requests_handler_impl::RequestsHandlerImpl;
use server::testing_server_wrapper;
use server::testing_server_wrapper::ServerWrapper;
use testing_utils::testing_config;

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
    serde_json::from_str(&response).unwrap_or_else(|_| {
        panic!(
            "Expected JSON response for query: {}, got: {}",
            url, response
        )
    })
}

fn assert_status_ok(response: &JsonValue) {
    assert_status(response, constants::FIELD_STATUS_OK);
}

fn assert_status(response: &JsonValue, expected_status: &str) {
    let status = response[constants::FIELD_NAME_STATUS]
        .as_str()
        .unwrap_or_else(|| panic!("Response must have status, but it didn't: {}", response));
    if status != expected_status {
        panic!("{} != {}, response: {}", expected_status, status, response)
    }
}

/// Cleaning up before tests
fn delete_app_user_with(uid: &Uuid) {
    use db::core::util::delete_app_user;
    delete_app_user(&uid, &testing_connection_for_server_user().unwrap()).unwrap();
}

#[test]
fn register_client_cmd_with_vk_user() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000000").unwrap();
    delete_app_user_with(&uid);

    let vk_override = r#"
    {
      "success": 1,
      "user_id": "uid1",
      "date": 123,
      "expire": 1234
    }"#;

    let override_str = create_vk_overrides(&uid, &vk_override);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    // Make sure we received a client token and it's a valid UUID
    let client_token = response[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    Uuid::parse_str(client_token).unwrap();

    // Make sure app_user and vk_user were created
    let conn = testing_connection_for_server_user().unwrap();
    let app_user = app_user::select_by_uid(&uid, &conn).unwrap().unwrap();
    assert_eq!("name1", app_user.name());
    let vk_user = vk_user::select_by_vk_uid("uid1", &conn).unwrap().unwrap();
    assert_eq!("uid1", vk_user.vk_uid());

    // Make sure gp_user is not created
    let gp_user = gp_user::select_by_gp_uid("uid1", &conn).unwrap();
    assert!(gp_user.is_none());
}

#[test]
fn registration_with_real_vk_token_fails_because_token_is_invalid() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000001").unwrap();
    delete_app_user_with(&uid);

    let override_str = create_uuid_only_overrides(&uid);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "INVALIDTOKEN",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_TOKEN_CHECK_FAIL);
}

#[test]
fn register_client_cmd_with_gp_user() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000002").unwrap();
    delete_app_user_with(&uid);

    let gp_override = r#"
    {
      "sub": "gp_uid1"
    }"#;

    let override_str = create_gp_overrides(&uid, &gp_override);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
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

    // Make sure we received a client token and it's a valid UUID
    let client_token = response[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    Uuid::parse_str(client_token).unwrap();

    // Make sure app_user and gp_user were created
    let conn = testing_connection_for_server_user().unwrap();
    let app_user = app_user::select_by_uid(&uid, &conn).unwrap().unwrap();
    assert_eq!("name1", app_user.name());
    let gp_user = gp_user::select_by_gp_uid("gp_uid1", &conn)
        .unwrap()
        .unwrap();
    assert_eq!("gp_uid1", gp_user.gp_uid());

    // Make sure vk_user is not created
    let vk_user = vk_user::select_by_vk_uid("gp_uid1", &conn).unwrap();
    assert!(vk_user.is_none());
}

#[test]
fn registration_with_real_gp_token_fails_because_token_is_invalid() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000003").unwrap();
    delete_app_user_with(&uid);

    let override_str = create_uuid_only_overrides(&uid);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "INVALIDTOKEN",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_TOKEN_CHECK_FAIL);
}

#[test]
fn user_registration_returns_duplication_error_on_uid_duplication() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000004").unwrap();
    delete_app_user_with(&uid);

    let vk_override = r#"
    {
      "success": 1,
      "user_id": "uid2",
      "date": 123,
      "expire": 1234
    }"#;
    let override_str = create_vk_overrides(&uid, &vk_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name2",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    let vk_override = r#"
    {
      "success": 1,
      "user_id": "uid3",
      "date": 123,
      "expire": 1234
    }"#;
    let override_str = create_vk_overrides(&uid, &vk_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name3",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_INTERNAL_ERROR);

    // Assert vk user not created
    let conn = testing_connection_for_server_user().unwrap();
    let vk_user = vk_user::select_by_vk_uid("uid3", &conn).unwrap();
    assert!(vk_user.is_none());
}

#[test]
fn register_client_fails_when_no_social_network_type_provided() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000005").unwrap();
    delete_app_user_with(&uid);

    let override_str = create_uuid_only_overrides(&uid);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);

    let app_user =
        app_user::select_by_uid(&uid, &testing_connection_for_server_user().unwrap()).unwrap();
    assert!(app_user.is_none());
}

#[test]
fn register_client_fails_when_no_social_network_token_provided() {
    let server = start_server();

    let uid = Uuid::from_str("00000000-b000-0000-0000-000000000006").unwrap();
    delete_app_user_with(&uid);

    let override_str = create_uuid_only_overrides(&uid);

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name1",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "type",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);

    let app_user =
        app_user::select_by_uid(&uid, &testing_connection_for_server_user().unwrap()).unwrap();
    assert!(app_user.is_none());
}

#[test]
fn vk_uid_duplication_returns_duplication_error() {
    let server = start_server();

    let uid1 = Uuid::from_str("00000000-b000-0000-0000-000000000007").unwrap();
    let uid2 = Uuid::from_str("00000000-b000-0000-0000-000000000008").unwrap();
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let duplicated_vk_override = r#"
    {
      "success": 1,
      "user_id": "uid4",
      "date": 123,
      "expire": 1234
    }"#;
    let override_str = create_vk_overrides(&uid1, &duplicated_vk_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name2",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    let override_str = create_vk_overrides(&uid2, &duplicated_vk_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name3",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "vk",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_ALREADY_REGISTERED);
}

#[test]
fn gp_uid_duplication_returns_duplication_error() {
    let server = start_server();

    let uid1 = Uuid::from_str("00000000-b000-0000-0000-000000000009").unwrap();
    let uid2 = Uuid::from_str("00000000-b000-0000-0000-000000000010").unwrap();
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let duplicated_gp_override = r#"
    {
      "sub": "gp_uid2"
    }"#;
    let override_str = create_gp_overrides(&uid1, &duplicated_gp_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name2",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    let override_str = create_gp_overrides(&uid2, &duplicated_gp_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_REGISTER_USER,
        &constants::ARG_USER_NAME,
        "name3",
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_ALREADY_REGISTERED);
}
