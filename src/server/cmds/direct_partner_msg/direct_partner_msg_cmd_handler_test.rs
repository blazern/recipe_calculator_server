use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use crate::server::constants;
use crate::server::testing_hostname;
use crate::server::testing_mock_server::FullRequest;

use crate::server::cmds::direct_partner_msg::direct_partner_msg_cmd_handler::insert_construction_overrides;

use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::make_request_with_body;
use crate::server::cmds::testing_cmds_utils::pair;
use crate::server::cmds::testing_cmds_utils::register_named_user_return_token;
use crate::server::cmds::testing_cmds_utils::set_user_fcm_token;
use crate::server::cmds::testing_cmds_utils::start_mock_server;
use crate::server::cmds::testing_cmds_utils::start_server_with_overrides;
use crate::server::cmds::testing_cmds_utils::{assert_status, assert_status_ok};

#[test]
fn pairing_with_fcm() {
    let r = |_request: &FullRequest| {
        let response = r#"
        {
            "multicast_id":2513734409441993719,
            "success":1,
            "failure":0,
            "canonical_ids":0,
            "results":[{"message_id":"0:1579970411599831%8e9256aef9fd7ecd"}]
        }"#;
        Some(response.to_owned())
    };
    let (fcm_server, fcm_requests) = start_mock_server(r, testing_hostname::get_spare_hostname1());

    let mut overrides = json!({});
    let fcm_addr = format!("http://{}", fcm_server.address());
    insert_construction_overrides(&mut overrides, fcm_addr);
    let server = start_server_with_overrides(&overrides);

    let uid1 = Uuid::from_str("00000000-d101-0000-0000-000000000000").unwrap();
    let uid2 = Uuid::from_str("00000000-d101-0000-0000-000000000001").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let client_token1 = register_named_user_return_token(server.address(), &uid1, &gpuid1, "name1");
    let client_token2 = register_named_user_return_token(server.address(), &uid2, &gpuid2, "name2");

    set_user_fcm_token(
        server.address(),
        &client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    pair(
        server.address(),
        &client_token1,
        &uid1.to_string(),
        &client_token2,
        &uid2.to_string(),
    );

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_DIRECT_PARTNER_MSG,
        &constants::ARG_USER_ID,
        percent_encode(&uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let msg = "That is my msg";
    let response = make_request_with_body(&url, msg.to_owned());
    assert_status_ok(&response);

    let fcm_requests = fcm_requests.lock().unwrap();
    let fcm_requests: Vec<JsonValue> = fcm_requests
        .iter()
        .map(|req| serde_json::from_str(&req.body).unwrap())
        .collect();

    assert_eq!(1, fcm_requests.len());

    assert_eq!(fcm_requests[0]["to"], json!(fcm_token2));
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_MSG_TYPE],
        constants::SERV_MSG_DIRECT_MSG_FROM_PARTNER,
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PARTNER_USER_ID],
        &uid1.to_string()
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PARTNER_NAME],
        "name1"
    );
    assert_eq!(&fcm_requests[0]["data"][constants::SERV_FIELD_MSG], &msg);
}

#[test]
fn non_existing_partner_not_found() {
    let r = |_request: &FullRequest| Some("".to_owned());
    let (fcm_server, fcm_requests) = start_mock_server(r, testing_hostname::get_spare_hostname1());

    let mut overrides = json!({});
    let fcm_addr = format!("http://{}", fcm_server.address());
    insert_construction_overrides(&mut overrides, fcm_addr);
    let server = start_server_with_overrides(&overrides);

    let uid1 = Uuid::from_str("00000000-d101-0000-0000-000000000002").unwrap();
    let uid2 = Uuid::from_str("00000000-d101-0000-0000-000000000003").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let client_token1 = register_named_user_return_token(server.address(), &uid1, &gpuid1, "name1");
    let client_token2 = register_named_user_return_token(server.address(), &uid2, &gpuid2, "name2");

    set_user_fcm_token(
        server.address(),
        &client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    pair(
        server.address(),
        &client_token1,
        &uid1.to_string(),
        &client_token2,
        &uid2.to_string(),
    );

    let invalid_uid2 = Uuid::from_str("00000000-d101-0000-0000-000000000004").unwrap();

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_DIRECT_PARTNER_MSG,
        &constants::ARG_USER_ID,
        percent_encode(&uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&invalid_uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let msg = "That is my msg";
    let response = make_request_with_body(&url, msg.to_owned());
    assert_status(&response, constants::FIELD_STATUS_PARTNER_USER_NOT_FOUND);

    let fcm_requests = fcm_requests.lock().unwrap();
    let fcm_requests: Vec<JsonValue> = fcm_requests
        .iter()
        .map(|req| serde_json::from_str(&req.body).unwrap())
        .collect();
    assert_eq!(0, fcm_requests.len());
}

#[test]
fn user_which_is_not_partner_is_not_found_as_partner() {
    let r = |_request: &FullRequest| Some("".to_owned());
    let (fcm_server, fcm_requests) = start_mock_server(r, testing_hostname::get_spare_hostname1());

    let mut overrides = json!({});
    let fcm_addr = format!("http://{}", fcm_server.address());
    insert_construction_overrides(&mut overrides, fcm_addr);
    let server = start_server_with_overrides(&overrides);

    let uid1 = Uuid::from_str("00000000-d101-0000-0000-000000000005").unwrap();
    let uid2 = Uuid::from_str("00000000-d101-0000-0000-000000000006").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let client_token1 = register_named_user_return_token(server.address(), &uid1, &gpuid1, "name1");
    let client_token2 = register_named_user_return_token(server.address(), &uid2, &gpuid2, "name2");

    set_user_fcm_token(
        server.address(),
        &client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    // Users are NOT partners
    //    pair(server.address(), &client_token1, &uid1.to_string(), &client_token2, &uid2.to_string());

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_DIRECT_PARTNER_MSG,
        &constants::ARG_USER_ID,
        percent_encode(&uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let msg = "That is my msg";
    let response = make_request_with_body(&url, msg.to_owned());
    assert_status(&response, constants::FIELD_STATUS_PARTNER_USER_NOT_FOUND);

    let fcm_requests = fcm_requests.lock().unwrap();
    let fcm_requests: Vec<JsonValue> = fcm_requests
        .iter()
        .map(|req| serde_json::from_str(&req.body).unwrap())
        .collect();
    assert_eq!(0, fcm_requests.len());
}
