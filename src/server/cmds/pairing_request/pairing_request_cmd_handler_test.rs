use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::paired_partners;
use crate::db::core::testing_util::testing_connection_for_server_user;
use crate::server::constants;
use crate::server::testing_hostname;
use crate::server::testing_mock_server::FullRequest;

use crate::server::cmds::pairing_request::pairing_request_cmd_handler;

use crate::server::cmds::testing_cmds_utils::assert_status;
use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::make_request;
use crate::server::cmds::testing_cmds_utils::pairing_request_by_code;
use crate::server::cmds::testing_cmds_utils::pairing_request_by_uid;
use crate::server::cmds::testing_cmds_utils::pairing_request_by_uid_with_overrides;
use crate::server::cmds::testing_cmds_utils::register_named_user;
use crate::server::cmds::testing_cmds_utils::register_user;
use crate::server::cmds::testing_cmds_utils::set_user_fcm_token;
use crate::server::cmds::testing_cmds_utils::start_mock_server;
use crate::server::cmds::testing_cmds_utils::start_pairing;

#[test]
fn pairing_by_pairing_codes() {
    let server = start_server!();

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000000").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000001").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_user(server.address(), &uid1, &gpuid1);
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_user(server.address(), &uid2, &gpuid2);
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let pairing_resp = start_pairing(server.address(), client_token1, &uid1.to_string());
    let pairing_code1 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    let pairing_resp = start_pairing(server.address(), client_token2, &uid2.to_string());
    let pairing_code2 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();

    pairing_request_by_code(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &pairing_code2,
    );
    pairing_request_by_code(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &pairing_code1,
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

#[test]
fn pairing_by_uids() {
    let server = start_server!();

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000002").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000003").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_user(server.address(), &uid1, &gpuid1);
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_user(server.address(), &uid2, &gpuid2);
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    pairing_request_by_uid(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &uid2.to_string(),
    );
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

#[test]
fn pairing_by_uid_and_pairing_code() {
    let server = start_server!();

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000004").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000005").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_user(server.address(), &uid1, &gpuid1);
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_user(server.address(), &uid2, &gpuid2);
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let pairing_resp = start_pairing(server.address(), client_token1, &uid1.to_string());
    let pairing_code1 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    start_pairing(server.address(), client_token2, &uid2.to_string());

    pairing_request_by_uid(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &uid2.to_string(),
    );
    pairing_request_by_code(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &pairing_code1,
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

#[test]
fn pairing_by_pairing_code_and_uid() {
    let server = start_server!();

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000006").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000007").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_user(server.address(), &uid1, &gpuid1);
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_user(server.address(), &uid2, &gpuid2);
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    let pairing_resp = start_pairing(server.address(), client_token2, &uid2.to_string());
    let pairing_code2 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();

    pairing_request_by_code(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &pairing_code2,
    );
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

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
    let server = start_server!(|overrides| {
        let fcm_addr = format!("http://{}", fcm_server.address());
        pairing_request_cmd_handler::insert_pairing_request_fcm_address_override(
            overrides, fcm_addr,
        );
    });

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000008").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000009").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token1 = format!("{}{}", uid1, "fcmtoken1");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    set_user_fcm_token(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &fcm_token1,
    );
    set_user_fcm_token(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    let pairing_resp = start_pairing(server.address(), client_token1, &uid1.to_string());
    let _pairing_code1 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    let pairing_resp = start_pairing(server.address(), client_token2, &uid2.to_string());
    let pairing_code2 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();

    // Sends request0 (see below)
    pairing_request_by_code(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &pairing_code2,
    );
    // Sends request1 and request2 (see below)
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );

    let fcm_requests = fcm_requests.lock().unwrap();
    let fcm_requests: Vec<JsonValue> = fcm_requests
        .iter()
        .map(|req| serde_json::from_str(&req.body).unwrap())
        .collect();

    // request0
    assert_eq!(fcm_requests[0]["to"], json!(fcm_token2));
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_MSG_TYPE],
        constants::SERV_MSG_PAIRING_REQUEST_FROM_PARTNER
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PAIRING_PARTNER_USER_ID],
        &uid1.to_string()
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PARTNER_NAME],
        "name1"
    );
    assert!(
        fcm_requests[0]["data"][constants::SERV_FIELD_REQUEST_EXPIRATION_DATE]
            .as_i64()
            .unwrap()
            > 0
    );

    // request1
    assert_eq!(fcm_requests[1]["to"], json!(fcm_token2));
    assert_eq!(
        &fcm_requests[1]["data"][constants::SERV_FIELD_MSG_TYPE],
        constants::SERV_MSG_PAIRED_WITH_PARTNER
    );
    assert_eq!(
        &fcm_requests[1]["data"][constants::SERV_FIELD_PAIRING_PARTNER_USER_ID],
        &uid1.to_string()
    );
    assert_eq!(
        &fcm_requests[1]["data"][constants::SERV_FIELD_PARTNER_NAME],
        "name1"
    );

    // request2
    assert_eq!(fcm_requests[2]["to"], json!(fcm_token1));
    assert_eq!(
        &fcm_requests[2]["data"][constants::SERV_FIELD_MSG_TYPE],
        constants::SERV_MSG_PAIRED_WITH_PARTNER
    );
    assert_eq!(
        &fcm_requests[2]["data"][constants::SERV_FIELD_PAIRING_PARTNER_USER_ID],
        &uid2.to_string()
    );
    assert_eq!(
        &fcm_requests[2]["data"][constants::SERV_FIELD_PARTNER_NAME],
        "name2"
    );
}

#[test]
fn real_invalid_fcm_tokens_do_not_cause_pairing_fail() {
    let server = start_server!();
    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000010").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000011").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token1 = format!("{}{}", uid1, "fcmtoken1");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    // NOTE: we set FCM tokens and real FCM servers are used - the tokens are invalid and
    // the FCM server will respond with errors.
    set_user_fcm_token(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &fcm_token1,
    );
    set_user_fcm_token(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    let pairing_resp = start_pairing(server.address(), client_token1, &uid1.to_string());
    let _pairing_code1 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    let pairing_resp = start_pairing(server.address(), client_token2, &uid2.to_string());
    let pairing_code2 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();

    pairing_request_by_code(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &pairing_code2,
    );
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

#[test]
fn invalid_fcm_tokens_do_not_cause_pairing_fail() {
    let r = |_request: &FullRequest| {
        // NOTE: InvalidRegistration
        let response = r#"
        {
            "multicast_id":3422611807746461474,
            "success":0,
            "failure":1,
            "canonical_ids":0,
            "results":[{"error":"InvalidRegistration"}]
        }"#;
        Some(response.to_owned())
    };
    let (fcm_server, _fcm_requests) = start_mock_server(r, testing_hostname::get_spare_hostname1());
    let server = start_server!(|overrides| {
        let fcm_addr = format!("http://{}", fcm_server.address());
        pairing_request_cmd_handler::insert_pairing_request_fcm_address_override(
            overrides, fcm_addr,
        );
    });

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000012").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000013").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token1 = format!("{}{}", uid1, "fcmtoken1");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    set_user_fcm_token(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &fcm_token1,
    );
    set_user_fcm_token(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    let pairing_resp = start_pairing(server.address(), client_token1, &uid1.to_string());
    let _pairing_code1 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    let pairing_resp = start_pairing(server.address(), client_token2, &uid2.to_string());
    let pairing_code2 = &pairing_resp[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();

    pairing_request_by_code(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &pairing_code2,
    );
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );
}

#[test]
fn pairing_when_already_paired() {
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
    let server = start_server!(|overrides| {
        let fcm_addr = format!("http://{}", fcm_server.address());
        pairing_request_cmd_handler::insert_pairing_request_fcm_address_override(
            overrides, fcm_addr,
        );
    });

    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000014").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000015").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    let fcm_token1 = format!("{}{}", uid1, "fcmtoken1");
    let fcm_token2 = format!("{}{}", uid2, "fcmtoken2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    set_user_fcm_token(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &fcm_token1,
    );
    set_user_fcm_token(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &fcm_token2,
    );

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    pairing_request_by_uid(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &uid2.to_string(),
    );
    pairing_request_by_uid(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_some());
    assert_eq!(
        paired_partners::PairingState::Done,
        pp.unwrap().pairing_state()
    );

    // Clearing not interesting requests
    fcm_requests.lock().unwrap().clear();

    // Sending a pairing request even though already paired
    pairing_request_by_uid(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &uid2.to_string(),
    );

    // Verifying that the user which requested pairing again has received a success message
    let fcm_requests = fcm_requests.lock().unwrap();
    let fcm_requests: Vec<JsonValue> = fcm_requests
        .iter()
        .map(|req| serde_json::from_str(&req.body).unwrap())
        .collect();
    assert_eq!(1, fcm_requests.len());
    assert_eq!(fcm_requests[0]["to"], json!(fcm_token1));
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_MSG_TYPE],
        constants::SERV_MSG_PAIRED_WITH_PARTNER
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PAIRING_PARTNER_USER_ID],
        &uid2.to_string()
    );
    assert_eq!(
        &fcm_requests[0]["data"][constants::SERV_FIELD_PARTNER_NAME],
        "name2"
    );
}

#[test]
fn pairing_when_first_partner_request_was_too_long_ago() {
    let server = start_server!();
    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000016").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000017").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    // Pairing requests with a huge time lapse
    let mut overrides = json!({});
    let now1 = 123;
    let now2 = now1 + pairing_request_cmd_handler::PAIRING_CONFIRMATION_EXPIRATION_DELAY_SECS * 2;
    pairing_request_cmd_handler::insert_cmd_now_override(&mut overrides, now1);
    pairing_request_by_uid_with_overrides(
        server.address(),
        client_token1,
        &uid1.to_string(),
        &uid2.to_string(),
        &overrides,
    );
    pairing_request_cmd_handler::insert_cmd_now_override(&mut overrides, now2);
    pairing_request_by_uid_with_overrides(
        server.address(),
        client_token2,
        &uid2.to_string(),
        &uid1.to_string(),
        &overrides,
    );

    let conn = testing_connection_for_server_user().unwrap();
    let user1 = app_user::select_by_uid(&uid1, &conn).unwrap().unwrap();
    let user2 = app_user::select_by_uid(&uid2, &conn).unwrap().unwrap();
    let pp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(pp.is_none());
}

#[test]
fn lack_of_partner_id_and_pairing_code() {
    let server = start_server!();
    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000018").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000019").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_PAIRING_REQUEST,
        &constants::ARG_USER_ID,
        percent_encode(&uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn lack_of_user_id() {
    let server = start_server!();
    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000020").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000021").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_PAIRING_REQUEST,
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn lack_of_client_token() {
    let server = start_server!();
    let uid1 = Uuid::from_str("00000000-d100-0000-0000-000000000020").unwrap();
    let uid2 = Uuid::from_str("00000000-d100-0000-0000-000000000021").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid1");
    let gpuid2 = format!("{}{}", uid2, "gpuid2");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let reg_resp = register_named_user(server.address(), &uid1, &gpuid1, "name1");
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let reg_resp = register_named_user(server.address(), &uid2, &gpuid2, "name2");
    let client_token2 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    start_pairing(server.address(), client_token1, &uid1.to_string());
    start_pairing(server.address(), client_token2, &uid2.to_string());

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_PAIRING_REQUEST,
        &constants::ARG_USER_ID,
        percent_encode(&uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(&uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
}
