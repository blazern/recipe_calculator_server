extern crate serde_json;

use std::sync::Arc;

use outside::error::Error;
use outside::error::ErrorKind;
use outside::fcm;
use outside::http_client::HttpClient;
use testing_utils::exhaust_future;
use testing_utils::testing_config;

#[test]
fn send_with_invalid_client_token() {
    let user_fcm_token = "asdasd";
    let config = testing_config();

    let http_client = Arc::new(HttpClient::new().unwrap());

    // Note that we can't get a user token from a test -
    // GoogleApi doesn't give mock user tokens and doesn't provide
    // a way to auth in tests.

    let data = json!({});
    let send_result = fcm::send(
        &data,
        user_fcm_token,
        config.fcm_server_token(),
        http_client,
    );
    let send_result = exhaust_future(send_result).unwrap();

    match send_result {
        fcm::SendResult::Error(error) => {
            assert!(error.contains("InvalidRegistration"), error);
        }
        _ => panic!(
            "Expected send result to be err, but it was: {:?}",
            send_result
        ),
    }
}

#[test]
fn send_with_invalid_server_token() {
    let user_fcm_token = "asdasd1";
    let server_fcm_token = "asdasd2";

    let http_client = Arc::new(HttpClient::new().unwrap());

    // Note that we can't get a user token from a test -
    // GoogleApi doesn't give mock user tokens and doesn't provide
    // a way to auth in tests.

    let data = json!({});
    let send_result = fcm::send(&data, user_fcm_token, server_fcm_token, http_client);
    let send_result = exhaust_future(send_result).unwrap();

    match send_result {
        fcm::SendResult::Error(error) => {
            assert!(error.contains("Error 401"), error);
        }
        _ => panic!(
            "Expected send result to be err, but it was: {:?}",
            send_result
        ),
    }
}

#[test]
fn successful_send_response_is_parsed() {
    let response = r#"
    {
        "multicast_id":2513734409441993719,
        "success":1,
        "failure":0,
        "canonical_ids":0,
        "results":[{"message_id":"0:1579970411599831%8e9256aef9fd7ecd"}]
    }"#;

    let send_result = fcm::send_response_to_send_result(response.to_string()).unwrap();

    match send_result {
        fcm::SendResult::Success => {
            // ok
        }
        _ => panic!(
            "Expected send result to be success, but it was: {:?}",
            send_result
        ),
    }
}

#[test]
fn failed_send_response_is_parsed() {
    let response = r#"
    {
        "multicast_id":3422611807746461474,
        "success":0,
        "failure":1,
        "canonical_ids":0,
        "results":[{"error":"InvalidRegistration"}]
    }"#;

    let send_result = fcm::send_response_to_send_result(response.to_string()).unwrap();

    match send_result {
        fcm::SendResult::Success => panic!(
            "Expected send result to be failure, but it was: {:?}",
            send_result
        ),
        fcm::SendResult::Error(_) => {
            // ok
        }
    }
}

#[test]
fn invalid_server_token_response_is_parsed() {
    let response = r#"
        <HTML>
        <HEAD>
        <TITLE>The request&#39;s Authentication (Server-) Key contained an invalid or malformed FCM-Token (a.k.a. IID-Token).</TITLE>
        </HEAD>
        <BODY>
        <H1>The request&#39;s Authentication (Server-) Key contained an invalid or malformed FCM-Token (a.k.a. IID-Token).</H1>
        <H2>Error 401</H2>
        </BODY>
        </HTML>
        "#;

    let send_result = fcm::send_response_to_send_result(response.to_string()).unwrap();

    match send_result {
        fcm::SendResult::Success => panic!(
            "Expected send result to be failure, but it was: {:?}",
            send_result
        ),
        fcm::SendResult::Error(_) => {
            // ok
        }
    }
}

#[test]
fn unexpected_html_error_transforms_into_unexpected_response_error() {
    let response = r#"
        <HTML>
        <HEAD>
        <TITLE>Not found!</TITLE>
        </HEAD>
        <BODY>
        <H1>Not found!.</H1>
        <H2>Error 404</H2>
        </BODY>
        </HTML>
        "#;

    let send_result = fcm::send_response_to_send_result(response.to_string());
    match send_result {
        Err(Error(ErrorKind::UnexpectedResponseFormat(_), _)) => {
            // Ok
        }
        _ => panic!("Unexpected result: {:?}", send_result),
    }
}

#[test]
fn empty_results_transforms_into_unexpected_response_error() {
    let response = r#"
    {
        "multicast_id":2513734409441993719,
        "success":1,
        "failure":0,
        "canonical_ids":0,
        "results":[]
    }"#;

    let send_result = fcm::send_response_to_send_result(response.to_string());
    match send_result {
        Err(Error(ErrorKind::UnexpectedResponseFormat(_), _)) => {
            // Ok
        }
        _ => panic!("Unexpected result: {:?}", send_result),
    }
}

#[test]
fn multiple_results_transforms_into_unexpected_response_error() {
    let response = r#"
    {
        "multicast_id":2513734409441993719,
        "success":1,
        "failure":0,
        "canonical_ids":0,
        "results":[
            {"message_id":"0:1579970411599831%8e9256aef9fd7ecd"},
            {"message_id":"0:1579970411599831%8e9256aef9fd7ece"}
        ]
    }"#;

    let send_result = fcm::send_response_to_send_result(response.to_string());
    match send_result {
        Err(Error(ErrorKind::UnexpectedResponseFormat(_), _)) => {
            // Ok
        }
        _ => panic!("Unexpected result: {:?}", send_result),
    }
}

#[test]
fn lack_of_results_field_transforms_into_unexpected_response_error() {
    let response = r#"
    {
        "multicast_id":2513734409441993719,
        "success":1,
        "failure":0,
        "canonical_ids":0
    }"#;

    let send_result = fcm::send_response_to_send_result(response.to_string());
    match send_result {
        Err(Error(ErrorKind::UnexpectedResponseFormat(_), _)) => {
            // Ok
        }
        _ => panic!("Unexpected result: {:?}", send_result),
    }
}
