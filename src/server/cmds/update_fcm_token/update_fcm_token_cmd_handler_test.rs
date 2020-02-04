use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::fcm_token;
use crate::db::core::testing_util::testing_connection_for_server_user;
use crate::server::constants;
use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;

use crate::server::cmds::testing_cmds_utils::assert_status;
use crate::server::cmds::testing_cmds_utils::assert_status_ok;
use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::make_request;
use crate::server::cmds::testing_cmds_utils::register_user;

#[test]
fn by_default_there_is_no_fcm_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d000-0000-0000-000000000000").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid0");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    assert_status_ok(&reg_resp);

    let conn = testing_connection_for_server_user().unwrap();
    let user = app_user::select_by_uid(&uid, &conn).unwrap().unwrap();
    let fcm_token = fcm_token::select_by_user_id(user.id(), &conn).unwrap();
    assert!(fcm_token.is_none());
}

#[test]
fn set_and_update_fcm_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d000-0000-0000-000000000001").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid1");
    let fcm_token0 = format!("{}{}", uid, "fcm0");
    let fcm_token1 = format!("{}{}", uid, "fcm1");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    let client_token = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    // Set
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_FCM_TOKEN,
        &constants::ARG_USER_ID,
        percent_encode(uid.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_FCM_TOKEN,
        percent_encode(&fcm_token0.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status_ok(&resp);

    // Check
    let conn = testing_connection_for_server_user().unwrap();
    let user = app_user::select_by_uid(&uid, &conn).unwrap().unwrap();
    let db_fcm_token = fcm_token::select_by_user_id(user.id(), &conn)
        .unwrap()
        .unwrap();
    assert_eq!(fcm_token0, db_fcm_token.token_value());

    // Update
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_FCM_TOKEN,
        &constants::ARG_USER_ID,
        percent_encode(uid.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_FCM_TOKEN,
        percent_encode(&fcm_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status_ok(&resp);

    // Check again
    let db_fcm_token = fcm_token::select_by_user_id(user.id(), &conn)
        .unwrap()
        .unwrap();
    assert_eq!(fcm_token1, db_fcm_token.token_value());
}

#[test]
fn lack_of_user_id() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d000-0000-0000-000000000002").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid2");
    let fcm_token = format!("{}{}", uid, "fcm2");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    let client_token = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_FCM_TOKEN,
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_FCM_TOKEN,
        percent_encode(&fcm_token.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status(&resp, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn lack_of_client_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d000-0000-0000-000000000003").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid3");
    let fcm_token = format!("{}{}", uid, "fcm3");
    delete_app_user_with(&uid);

    let _reg_resp = register_user(server.address(), &uid, &gpuid);

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_FCM_TOKEN,
        &constants::ARG_USER_ID,
        percent_encode(uid.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_FCM_TOKEN,
        percent_encode(&fcm_token.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status(&resp, constants::FIELD_STATUS_PARAM_MISSING);
}
