extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::diesel_connection;
use db::core::app_user::app_user as app_user_schema;
use db::core::testing_util as dbtesting_utils;

// Cleaning up before tests
fn delete_entry_with(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);
    delete_by_column!(app_user_schema::table, app_user_schema::uid, uid, raw_connection)
        .expect("Deletion shouldn't fail");
    let deleted_user =
        select_by_column!(app_user::AppUser, app_user_schema::table, app_user_schema::uid, uid, raw_connection);
    assert!(deleted_user.expect("Selection shouldn't fail").is_none());
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-009000000000").unwrap();
    delete_entry_with(&uid);

    let new_user = app_user::new(uid);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted_user = app_user::insert(new_user, &connection).unwrap();
    assert!(inserted_user.id() > 0);
    assert_eq!(*inserted_user.uid(), uid);

    let selected_user = app_user::select_by_id(inserted_user.id(), &connection);
    let selected_user = selected_user.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_user, selected_user);
}

#[test]
fn cant_insert_user_with_already_used_uid() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-009000000001").unwrap();
    delete_entry_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = app_user::new(uid);
    let user2 = app_user::new(uid);

    app_user::insert(user1, &connection).unwrap();

    let second_insertion_result = app_user::insert(user2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn can_select_user_by_uid() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-009000000002").unwrap();
    delete_entry_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted_user = app_user::insert(app_user::new(uid), &connection).unwrap();

    let selected_user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();

    assert_eq!(inserted_user, selected_user);
}

#[test]
fn can_delete_user_by_id() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-009000000003").unwrap();
    delete_entry_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let inserted_user = app_user::insert(app_user::new(uid), &connection).unwrap();

    app_user::delete_by_id(inserted_user.id(), &connection).unwrap();
    let deleted_user = app_user::select_by_uid(&uid, &connection).unwrap();

    assert!(deleted_user.is_none());
}

#[test]
fn cant_delete_user_with_client_connection() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-009000000004").unwrap();
    delete_entry_with(&uid);

    let pg_client_connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted_user = app_user::insert(app_user::new(uid), &pg_client_connection).unwrap();

    let user_deletion_result = app_user::delete_by_id(inserted_user.id(), &pg_client_connection);

    assert!(user_deletion_result.is_err());
}
