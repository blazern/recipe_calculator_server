extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::diesel_connection;
use db::core::testing_util as dbtesting_utils;
use db::core::vk_user;
use db::core::vk_user::vk_user as vk_user_schema;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    testing_util_delete_entries_with!(
        app_user_uid,
        vk_user_schema::table,
        vk_user_schema::app_user_id);
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let vk_uid = 1;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002000000000").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_vk_user = vk_user::new(vk_uid, &app_user);

    let inserted_vk_user = vk_user::insert(new_vk_user, &connection).unwrap();
    assert!(inserted_vk_user.id() > 0);
    assert_eq!(inserted_vk_user.vk_uid(), vk_uid);
    assert_eq!(app_user.id(), inserted_vk_user.app_user_id());

    let selected_vk_user = vk_user::select_by_id(inserted_vk_user.id(), &connection);
    let selected_vk_user = selected_vk_user.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_vk_user, selected_vk_user);
}

#[test]
fn cant_insert_vk_user_with_already_used_vk_uid() {
    let vk_uid = 2;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002000000001").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let vk_user_copy1 = vk_user::new(vk_uid, &app_user);
    let vk_user_copy2 = vk_user::new(vk_uid, &app_user);

    vk_user::insert(vk_user_copy1, &connection).unwrap();

    let second_insertion_result = vk_user::insert(vk_user_copy2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn multiple_vk_users_cannot_depend_on_single_app_user() {
    let vk_uid1 = 3;
    let vk_uid2 = 4;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-002000000002").unwrap();
    delete_entries_with(&app_user_uid);


    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let vk_user1 = vk_user::new(vk_uid1, &app_user);
    let vk_user2 = vk_user::new(vk_uid2, &app_user);

    vk_user::insert(vk_user1, &connection).unwrap();

    let second_user_selection_result = vk_user::insert(vk_user2, &connection);
    assert!(second_user_selection_result.is_err());
}