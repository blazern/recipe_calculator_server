use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
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
fn update_user_name() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d001-0000-0000-000000000001").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid1");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    let client_token = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let connection = testing_connection_for_server_user().unwrap();
    let initial_user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    let new_name = initial_user.name().to_owned() + "updated";

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_USER_NAME,
        &constants::ARG_USER_ID,
        percent_encode(uid.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_USER_NAME,
        percent_encode(&new_name.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status_ok(&resp);

    let updated_user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    assert_ne!(initial_user, updated_user);
    assert_eq!(new_name, updated_user.name());
}

#[test]
fn lack_of_user_id() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d001-0000-0000-000000000002").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid2");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    let client_token = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    let connection = testing_connection_for_server_user().unwrap();
    let initial_user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    let new_name = initial_user.name().to_owned() + "updated";

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_USER_NAME,
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&client_token.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_USER_NAME,
        percent_encode(&new_name.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status(&resp, constants::FIELD_STATUS_PARAM_MISSING);
}

#[test]
fn lack_of_client_token() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-d001-0000-0000-000000000003").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid3");
    delete_app_user_with(&uid);

    let _reg_resp = register_user(server.address(), &uid, &gpuid);

    let connection = testing_connection_for_server_user().unwrap();
    let initial_user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    let new_name = initial_user.name().to_owned() + "updated";

    let url = format!(
        "http://{}{}?{}={}&{}={}",
        server.address(),
        &constants::CMD_UPDATE_USER_NAME,
        &constants::ARG_USER_ID,
        percent_encode(uid.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_USER_NAME,
        percent_encode(&new_name.as_bytes(), DEFAULT_ENCODE_SET).to_string()
    );
    let resp = make_request(&url);
    assert_status(&resp, constants::FIELD_STATUS_PARAM_MISSING);
}
