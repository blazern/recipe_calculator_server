use serde_json::Value as JsonValue;
use std::str::FromStr;
use std::thread;
use std::time;
use uuid::Uuid;

use crate::server::constants;
use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;

use crate::server::cmds::start_pairing::start_pairing_cmd_handler::insert_pairing_code_gen_extended_override;
use crate::server::cmds::testing_cmds_utils::assert_status;
use crate::server::cmds::testing_cmds_utils::assert_status_ok;
use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::make_request;
use crate::server::cmds::testing_cmds_utils::register_user;
use crate::server::cmds::testing_cmds_utils::start_server_with_overrides;

use crate::utils::now_source::{DefaultNowSource, NowSource};

#[test]
fn start_pairing() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000000").unwrap();
    delete_app_user_with(&uid);

    let reg_resp1 = register_user(server.address(), &uid, "gpuid1");
    let now = DefaultNowSource {}.now_secs().unwrap();
    let client_token = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let response = start_pairing_request(
        server.address(),
        Some(&uid.to_string()),
        Some(&client_token),
    );
    assert_status_ok(&response);

    let code = response[constants::FIELD_NAME_PAIRING_CODE]
        .as_str()
        .unwrap();
    let code = code.parse::<i32>().unwrap();
    let expiration_date = response[constants::FIELD_NAME_PAIRING_CODE_EXPIRATION_DATE]
        .as_i64()
        .unwrap();

    assert!(0 <= code && code <= 9999, "Generated code: {}", code);
    assert!(now <= expiration_date);
}

#[test]
fn start_pairing_of_non_existing_user() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000001").unwrap();
    delete_app_user_with(&uid);

    let reg_resp1 = register_user(server.address(), &uid, "gpuid2");
    let client_token = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let uid2 = Uuid::from_str("00000000-c000-0000-0000-000000000002").unwrap();
    let response = start_pairing_request(
        server.address(),
        Some(&uid2.to_string()),
        Some(&client_token),
    );
    assert_status(&response, constants::FIELD_STATUS_USER_NOT_FOUND);
}

#[test]
fn wrong_client_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000003").unwrap();
    delete_app_user_with(&uid);

    register_user(server.address(), &uid, "gpuid3");
    let wrong_client_token = "00000000-c000-0000-0000-000000000003";
    let response = start_pairing_request(
        server.address(),
        Some(&uid.to_string()),
        Some(&wrong_client_token),
    );
    assert_status(&response, constants::FIELD_STATUS_INVALID_CLIENT_TOKEN);
}

#[test]
fn invalid_user_id() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000004").unwrap();
    delete_app_user_with(&uid);

    let reg_resp1 = register_user(server.address(), &uid, "gpuid4");
    let client_token = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let response = start_pairing_request(server.address(), Some("asd"), Some(&client_token));
    assert_status(&response, constants::FIELD_STATUS_INVALID_UUID);
}

#[test]
fn lack_of_client_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000005").unwrap();
    delete_app_user_with(&uid);

    register_user(server.address(), &uid, "gpuid5");

    let response = start_pairing_request(server.address(), Some(&uid.to_string()), None);
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn lack_of_user_id() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-c000-0000-0000-000000000006").unwrap();
    delete_app_user_with(&uid);

    let reg_resp1 = register_user(server.address(), &uid, "gpuid6");
    let client_token = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let response = start_pairing_request(server.address(), None, Some(&client_token));
    assert_status(&response, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn all_ids_taken() {
    let fam = format!("{}{}", file!(), line!());
    let mut overrides = json!({});
    let left = 0;
    let right = 2;
    let lifetime = 100;
    insert_pairing_code_gen_extended_override(&mut overrides, fam, left, right, lifetime, true);
    let server = start_server_with_overrides(&overrides);

    let uid1 = Uuid::from_str("00000000-c000-0000-0000-000000000007").unwrap();
    let uid2 = Uuid::from_str("00000000-c000-0000-0000-000000000008").unwrap();
    let uid3 = Uuid::from_str("00000000-c000-0000-0000-000000000009").unwrap();
    let uid4 = Uuid::from_str("00000000-c000-0000-0000-000000000010").unwrap();
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);
    delete_app_user_with(&uid3);
    delete_app_user_with(&uid4);

    let reg_resp1 = register_user(server.address(), &uid1, "gpuid7");
    let reg_resp2 = register_user(server.address(), &uid2, "gpuid8");
    let reg_resp3 = register_user(server.address(), &uid3, "gpuid9");
    let reg_resp4 = register_user(server.address(), &uid4, "gpuid10");

    let client_token1 = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let client_token2 = &reg_resp2[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let client_token3 = &reg_resp3[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let client_token4 = &reg_resp4[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let resp1 = start_pairing_request(
        server.address(),
        Some(&uid1.to_string()),
        Some(&client_token1),
    );
    let resp2 = start_pairing_request(
        server.address(),
        Some(&uid2.to_string()),
        Some(&client_token2),
    );
    let resp3 = start_pairing_request(
        server.address(),
        Some(&uid3.to_string()),
        Some(&client_token3),
    );
    let resp4 = start_pairing_request(
        server.address(),
        Some(&uid4.to_string()),
        Some(&client_token4),
    );

    assert_status_ok(&resp1);
    assert_status_ok(&resp2);
    assert_status_ok(&resp3);
    assert_status(&resp4, constants::FIELD_STATUS_INTERNAL_ERROR);
}

#[test]
fn all_ids_taken_and_freed_after_time() {
    let fam = format!("{}{}", file!(), line!());
    let mut overrides = json!({});
    let left = 0;
    let right = 1;
    let lifetime = 3;
    insert_pairing_code_gen_extended_override(&mut overrides, fam, left, right, lifetime, true);
    let server = start_server_with_overrides(&overrides);

    let uid1 = Uuid::from_str("00000000-c000-0000-0000-000000000011").unwrap();
    let uid2 = Uuid::from_str("00000000-c000-0000-0000-000000000012").unwrap();
    let uid3 = Uuid::from_str("00000000-c000-0000-0000-000000000013").unwrap();
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);
    delete_app_user_with(&uid3);

    let reg_resp1 = register_user(server.address(), &uid1, "gpuid11");
    let reg_resp2 = register_user(server.address(), &uid2, "gpuid12");
    let reg_resp3 = register_user(server.address(), &uid3, "gpuid13");

    let client_token1 = &reg_resp1[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let client_token2 = &reg_resp2[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let client_token3 = &reg_resp3[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let resp1 = start_pairing_request(
        server.address(),
        Some(&uid1.to_string()),
        Some(&client_token1),
    );
    let resp2 = start_pairing_request(
        server.address(),
        Some(&uid2.to_string()),
        Some(&client_token2),
    );
    let resp3 = start_pairing_request(
        server.address(),
        Some(&uid3.to_string()),
        Some(&client_token3),
    );

    assert_status_ok(&resp1);
    assert_status_ok(&resp2);

    // All codes taken
    assert_status(&resp3, constants::FIELD_STATUS_INTERNAL_ERROR);

    let lifetime = time::Duration::from_secs((lifetime + 1) as u64);
    thread::sleep(lifetime);
    // Now the codes are freed and the third user can get a code
    let resp3 = start_pairing_request(
        server.address(),
        Some(&uid3.to_string()),
        Some(&client_token3),
    );
    assert_status_ok(&resp3);
}

fn start_pairing_request(
    server_addr: &str,
    uid: Option<&str>,
    client_token: Option<&str>,
) -> JsonValue {
    let url = match (uid, client_token) {
        (Some(uid), Some(client_token)) => format!(
            "http://{}{}?{}={}&{}={}",
            server_addr,
            &constants::CMD_START_PAIRING,
            &constants::ARG_USER_ID,
            percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
            &constants::ARG_CLIENT_TOKEN,
            percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        ),
        (Some(uid), None) => format!(
            "http://{}{}?{}={}",
            server_addr,
            &constants::CMD_START_PAIRING,
            &constants::ARG_USER_ID,
            percent_encode(uid.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        ),
        (None, Some(client_token)) => format!(
            "http://{}{}?{}={}",
            server_addr,
            &constants::CMD_START_PAIRING,
            &constants::ARG_CLIENT_TOKEN,
            percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        ),
        (None, None) => format!("http://{}{}", server_addr, &constants::CMD_START_PAIRING,),
    };
    make_request(&url)
}
