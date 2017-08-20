extern crate recipe_calculator_server;
extern crate serde_json;

use recipe_calculator_server::vk;
use std::env;

const VK_SERVER_TOKEN_ENV_VAR_NAME: &'static str = "VK_SERVER_TOKEN";

#[test]
fn can_check_client_token() {
    let server_token = env::var(VK_SERVER_TOKEN_ENV_VAR_NAME);
    let server_token = match server_token {
        Ok(val) => val,
        Err(err) => panic!("Is env var {} provided? Error: {:?}", VK_SERVER_TOKEN_ENV_VAR_NAME, err),
    };
    let user_token = "asdasd";

    // Note that we can't get a user token from a test -
    // VK api doesn't give mock user tokens and doesn't provide
    // a way to auth in tests.
    let check_result = vk::check_token(&server_token, user_token).unwrap();

    assert!(!check_result.is_success());
    assert!(check_result.error_code().unwrap() == vk::ERROR_CODE_CLIENT_TOKEN_INVALID);
    assert!(!check_result.error_msg().as_ref().unwrap().is_empty());
    assert!(check_result.user_id().is_none());
}

#[test]
fn cant_check_client_token_if_server_token_invalid() {
    let server_token = "adsasd";
    let user_token = "asdasd";

    let check_result = vk::check_token(server_token, user_token).unwrap();

    assert!(!check_result.is_success());
    assert!(check_result.error_code().unwrap() == vk::ERROR_CODE_SERVER_TOKEN_INVALID);
    assert!(!check_result.error_msg().as_ref().unwrap().is_empty());
    assert!(check_result.user_id().is_none());
}

#[test]
fn successful_check_response_is_parsed() {
    let response = r#"
    {
      "success": 1,
      "user_id": "asd",
      "date": 123,
      "expire": 1234
    }"#;

    let check_result = vk::check_token_from_server_response(
        response.to_string().as_bytes()).unwrap();

    assert!(check_result.is_success());
    assert!(check_result.user_id().as_ref().unwrap() == "asd");
    assert!(check_result.error_code().is_none());
    assert!(check_result.error_msg().is_none());
}