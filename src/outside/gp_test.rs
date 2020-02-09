use std::sync::Arc;

use crate::outside::gp;
use crate::outside::http_client::HttpClient;
use crate::testing_utils::exhaust_future;

#[test]
fn can_check_client_token() {
    let user_token = "asdasd";

    let http_client = Arc::new(HttpClient::new().unwrap());

    // Note that we can't get a user token from a test -
    // GoogleApi doesn't give mock user tokens and doesn't provide
    // a way to auth in tests.

    let check_result = gp::check_token(user_token.to_owned(), http_client);
    let check_result = exhaust_future(check_result).unwrap();

    match check_result {
        gp::CheckResult::Error {
            error_title,
            error_descr,
        } => {
            assert_eq!(gp::ERROR_TITLE_CLIENT_TOKEN_INVALID, error_title);
            assert!(!error_descr.is_empty());
        }
        _ => panic!(
            "Expected check result to be err, but it was: {:?}",
            check_result
        ),
    }
}

#[test]
fn successful_check_response_is_parsed() {
    let response = r#"
    {
        "sub": "114152967454900866567"
    }"#;

    let check_result =
        gp::check_token_from_server_response(response.to_string().as_bytes()).unwrap();

    match check_result {
        gp::CheckResult::Success { user_id } => {
            assert_eq!("114152967454900866567", user_id);
        }
        _ => panic!(
            "Expected check result to be success, but it was: {:?}",
            check_result
        ),
    }
}

#[test]
fn failed_check_response_is_parsed() {
    let response = r#"
    {
        "error": "invalid_token",
        "error_description": "Invalid Value"
    }"#;

    let check_result =
        gp::check_token_from_server_response(response.to_string().as_bytes()).unwrap();

    match check_result {
        gp::CheckResult::Error {
            error_title,
            error_descr,
        } => {
            assert_eq!(gp::ERROR_TITLE_CLIENT_TOKEN_INVALID, error_title);
            assert_eq!("Invalid Value", error_descr);
        }
        _ => panic!(
            "Expected check result to be err, but it was: {:?}",
            check_result
        ),
    }
}
