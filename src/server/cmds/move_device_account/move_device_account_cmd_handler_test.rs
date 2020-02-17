use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::testing_util as dbtesting_utils;

use crate::server::cmds::register_user::user_data_generators::create_gp_overrides;
use crate::server::cmds::testing_cmds_utils::assert_status;
use crate::server::cmds::testing_cmds_utils::assert_status_ok;
use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::make_request;
use crate::server::cmds::testing_cmds_utils::register_user;
use crate::server::cmds::testing_cmds_utils::set_user_fcm_token_without_ok_check;
use crate::server::constants;

#[test]
fn move_device_account() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-e100-0000-0000-000000000000").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid");
    delete_app_user_with(&uid);

    let reg_resp = register_user(server.address(), &uid, &gpuid);
    let client_token1 = &reg_resp[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();

    // Same GP uid
    let gp_override = format!("{{ \"sub\": \"{}\" }}", gpuid);
    let override_str = create_gp_overrides(&uid, &gp_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_MOVE_DEVICE_ACCOUNT,
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    let client_token2 = &response[constants::FIELD_NAME_CLIENT_TOKEN]
        .as_str()
        .unwrap();
    let name = &response[constants::FIELD_NAME_USER_NAME].as_str().unwrap();
    let received_uid = &response[constants::FIELD_NAME_USER_ID].as_str().unwrap();

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    assert_eq!(user.name(), *name);
    assert_eq!(uid.to_string(), *received_uid);

    // No verify that old token became invalid and new token is valid
    let response = set_user_fcm_token_without_ok_check(
        server.address(),
        client_token1,
        &uid.to_string(),
        "asd",
    );
    assert_status(&response, constants::FIELD_STATUS_INVALID_CLIENT_TOKEN);

    let response = set_user_fcm_token_without_ok_check(
        server.address(),
        client_token2,
        &uid.to_string(),
        "asd",
    );
    assert_status_ok(&response);
}

#[test]
fn move_device_account_when_user_not_registered() {
    let server = start_server!();

    let uid = Uuid::from_str("00000000-e100-0000-0000-000000000000").unwrap();
    let gpuid = format!("{}{}", uid, "gpuid");
    delete_app_user_with(&uid);

    register_user(server.address(), &uid, &gpuid);

    // !!! Another GP uid
    let gp_override = format!("{{ \"sub\": \"{}{}\" }}", gpuid, "changed");
    let override_str = create_gp_overrides(&uid, &gp_override);
    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_MOVE_DEVICE_ACCOUNT,
        &constants::ARG_SOCIAL_NETWORK_TYPE,
        "gp",
        &constants::ARG_SOCIAL_NETWORK_TOKEN,
        "token",
        &constants::ARG_OVERRIDES,
        override_str
    );
    let response = make_request(&url);
    assert_status(&response, constants::FIELD_STATUS_USER_NOT_FOUND);
}
