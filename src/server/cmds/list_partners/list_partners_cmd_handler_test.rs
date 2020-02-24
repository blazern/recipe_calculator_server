use std::str::FromStr;
use uuid::Uuid;

use crate::server::cmds::testing_cmds_utils::delete_app_user_with;
use crate::server::cmds::testing_cmds_utils::list_partners;
use crate::server::cmds::testing_cmds_utils::pair;
use crate::server::cmds::testing_cmds_utils::register_named_user_return_token;
use crate::server::constants;

#[test]
fn list_partners_test() {
    let server = start_server!();

    let uid1 = Uuid::from_str("00000000-f100-0000-0000-000000000000").unwrap();
    let uid2 = Uuid::from_str("00000000-f100-0000-0000-000000000001").unwrap();
    let uid3 = Uuid::from_str("00000000-f100-0000-0000-000000000002").unwrap();
    let gpuid1 = format!("{}{}", uid1, "gpuid");
    let gpuid2 = format!("{}{}", uid2, "gpuid");
    let gpuid3 = format!("{}{}", uid3, "gpuid");
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);
    delete_app_user_with(&uid3);

    let client_token1 = register_named_user_return_token(server.address(), &uid1, &gpuid1, "name1");
    let client_token2 = register_named_user_return_token(server.address(), &uid2, &gpuid2, "name2");
    let client_token3 = register_named_user_return_token(server.address(), &uid3, &gpuid3, "name3");

    pair(
        server.address(),
        &client_token1,
        &uid1.to_string(),
        &client_token2,
        &uid2.to_string(),
    );
    pair(
        server.address(),
        &client_token3,
        &uid3.to_string(),
        &client_token1,
        &uid1.to_string(),
    );

    let response = list_partners(server.address(), &client_token1, &uid1.to_string());
    let partners_json = &response[constants::FIELD_NAME_PARTNERS];
    assert!(partners_json.is_array(), "{}", response);
    let partners = partners_json.as_array().unwrap();
    assert_eq!(2, partners.len());

    let partner2_json = json!({
        constants::FIELD_NAME_PARTNER_USER_ID: uid2.to_string(),
        constants::FIELD_NAME_PARTNER_NAME: "name2"
    });
    let partner3_json = json!({
        constants::FIELD_NAME_PARTNER_USER_ID: uid3.to_string(),
        constants::FIELD_NAME_PARTNER_NAME: "name3"
    });
    assert!(partners.contains(&partner2_json), "{}", response);
    assert!(partners.contains(&partner3_json), "{}", response);
}
