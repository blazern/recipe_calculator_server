extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::fcm_token;
use db::core::testing_util as dbtesting_utils;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    use db::core::util::delete_app_user;
    delete_app_user(
        &app_user_uid,
        &dbtesting_utils::testing_connection_for_server_user().unwrap(),
    )
    .unwrap();
}

// NOTE: different UUIDs and token values must be used in each tests, because tests are run in parallel
// and usage of same IDs and values would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let token_value = "1";
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002300000000").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_fcm_token = fcm_token::new(token_value.to_string(), &app_user);

    let inserted_fcm_token = fcm_token::insert(new_fcm_token, &connection).unwrap();
    assert!(inserted_fcm_token.id() > 0);
    assert_eq!(inserted_fcm_token.token_value(), token_value);
    assert_eq!(app_user.id(), inserted_fcm_token.app_user_id());

    let selected_fcm_token = fcm_token::select_by_id(inserted_fcm_token.id(), &connection);
    let selected_fcm_token = selected_fcm_token.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_fcm_token, selected_fcm_token);

    let selected_by_fcm_token = fcm_token::select_by_user_id(app_user.id(), &connection);
    let selected_by_fcm_token = selected_by_fcm_token.unwrap().unwrap();
    assert_eq!(inserted_fcm_token, selected_by_fcm_token);
}

#[test]
fn cant_insert_fcm_token_with_already_used_token_value() {
    let token_value = "2";
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002300000001").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let fcm_token_copy1 = fcm_token::new(token_value.to_string(), &app_user);
    let fcm_token_copy2 = fcm_token::new(token_value.to_string(), &app_user);

    fcm_token::insert(fcm_token_copy1, &connection).unwrap();

    let second_insertion_result = fcm_token::insert(fcm_token_copy2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn multiple_fcm_tokens_cannot_depend_on_single_app_user() {
    let token_value1 = "3";
    let token_value2 = "4";
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002300000002").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let fcm_token1 = fcm_token::new(token_value1.to_string(), &app_user);
    let fcm_token2 = fcm_token::new(token_value2.to_string(), &app_user);

    fcm_token::insert(fcm_token1, &connection).unwrap();

    let second_user_selection_result = fcm_token::insert(fcm_token2, &connection);
    assert!(second_user_selection_result.is_err());
}

#[test]
fn delete_by_user_id() {
    let token_value = "5";
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002300000003").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let fcm_token = fcm_token::new(token_value.to_string(), &app_user);
    fcm_token::insert(fcm_token, &connection).unwrap();

    assert!(fcm_token::select_by_user_id(app_user.id(), &connection)
        .unwrap()
        .is_some());
    fcm_token::delete_by_user_id(app_user.id(), &connection).unwrap();
    assert!(fcm_token::select_by_user_id(app_user.id(), &connection)
        .unwrap()
        .is_none());
}
