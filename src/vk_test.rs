extern crate serde_json;

use vk;

include!("testing_config.rs.inc");

#[test]
fn can_check_client_token() {
    let config = get_testing_config();
    let user_token = "asdasd";

    // Note that we can't get a user token from a test -
    // VK api doesn't give mock user tokens and doesn't provide
    // a way to auth in tests.
    let check_result = vk::check_token(&config.vk_server_token(), user_token).unwrap();

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