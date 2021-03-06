use percent_encoding::percent_encode;
use percent_encoding::DEFAULT_ENCODE_SET;
use std::str::FromStr;
use uuid::Uuid;

use crate::server::cmds::testing_cmds_utils::assert_status_ok;
use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::list_partners;
use crate::server::cmds::testing_cmds_utils::make_request;
use crate::server::cmds::testing_cmds_utils::pair;
use crate::server::cmds::testing_cmds_utils::register_named_user_return_token;
use crate::server::constants;

#[test]
fn unpair_test1() {
    let uid1 = Uuid::from_str("00000000-a200-0000-0000-000000000000").unwrap();
    let uid2 = Uuid::from_str("00000000-a200-0000-0000-000000000001").unwrap();
    unpair_test_impl(true, &uid1, &uid2);
}

#[test]
fn unpair_test2() {
    let uid1 = Uuid::from_str("00000000-a200-0000-0000-000000000002").unwrap();
    let uid2 = Uuid::from_str("00000000-a200-0000-0000-000000000003").unwrap();
    unpair_test_impl(false, &uid1, &uid2);
}

fn unpair_test_impl(direct_order: bool, uid1: &Uuid, uid2: &Uuid) {
    let server = start_server!();

    let gpuid1 = format!("{}{}", uid1, "gpuid");
    let gpuid2 = format!("{}{}", uid2, "gpuid");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let client_token1 = register_named_user_return_token(server.address(), &uid1, &gpuid1, "name1");
    let client_token2 = register_named_user_return_token(server.address(), &uid2, &gpuid2, "name2");

    pair(
        server.address(),
        &client_token1,
        &uid1.to_string(),
        &client_token2,
        &uid2.to_string(),
    );

    let response = list_partners(server.address(), &client_token1, &uid1.to_string());
    let partners_json = &response[constants::FIELD_NAME_PARTNERS];
    assert!(partners_json.is_array(), "{}", response);
    let partners = partners_json.as_array().unwrap();
    assert_eq!(1, partners.len());

    let (ordered_uid1, ordered_uid2) = if direct_order {
        (&uid1, &uid2)
    } else {
        (&uid2, &uid1)
    };
    let (ordered_client_token1, _ordered_client_token2) = if direct_order {
        (&client_token1, &client_token2)
    } else {
        (&client_token2, &client_token1)
    };

    let url = format!(
        "http://{}{}?{}={}&{}={}&{}={}",
        server.address(),
        &constants::CMD_UNPAIR,
        &constants::ARG_USER_ID,
        percent_encode(ordered_uid1.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_CLIENT_TOKEN,
        percent_encode(&ordered_client_token1.as_bytes(), DEFAULT_ENCODE_SET).to_string(),
        &constants::ARG_PARTNER_USER_ID,
        percent_encode(ordered_uid2.to_string().as_bytes(), DEFAULT_ENCODE_SET).to_string(),
    );
    let response = make_request(&url);
    assert_status_ok(&response);

    // Assert that user 1 no longer has partners
    let response = list_partners(server.address(), &client_token1, &uid1.to_string());
    let partners_json = &response[constants::FIELD_NAME_PARTNERS];
    assert!(partners_json.is_array(), "{}", response);
    let partners = partners_json.as_array().unwrap();
    assert_eq!(0, partners.len());

    // Assert that user 2 no longer has partners
    let response = list_partners(server.address(), &client_token2, &uid2.to_string());
    let partners_json = &response[constants::FIELD_NAME_PARTNERS];
    assert!(partners_json.is_array(), "{}", response);
    let partners = partners_json.as_array().unwrap();
    assert_eq!(0, partners.len());
}
