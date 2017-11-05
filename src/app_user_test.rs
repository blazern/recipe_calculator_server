extern crate diesel;
extern crate uuid;

use std::env;
use std::str::FromStr;
use diesel::Connection;
use diesel::ExecuteDsl;
use diesel::ExpressionMethods;
use diesel::FilterDsl;
use diesel::LoadDsl;
use diesel::pg::PgConnection;
use uuid::Uuid;

use app_user;
use schema;

include!("testing_config.rs.inc");

// Admin user has too many privileges and should be used only inside of tests.
const PSQL_ADMIN_URL: &'static str = "RECIPE_CALCULATOR_SERVER_PSQL_URL_USER_ADMIN";

// Cleaning up before tests
fn delete_entry_with(uid: &Uuid) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let pg_connection = PgConnection::establish(&psql_admin_url).unwrap();

    diesel::delete(
        schema::app_user::table.filter(
            schema::app_user::uid.eq(uid)))
        .execute(&pg_connection).unwrap();

    let all_users = schema::app_user::table
        .load::<app_user::AppUser>(&pg_connection).unwrap();

    for user in all_users {
        if user.uid() == uid {
            panic!("User with given uid is not removed! Looks like a race condition! Are IDs in different tests unique?");
        }
    }
}

// NOTE: different UUIDs and VK IDs mut be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let vk_uid = 1i32;
    delete_entry_with(&uid);

    let config = get_testing_config();
    let new_user = app_user::new(uid, vk_uid);
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let inserted_user = app_user::insert(new_user, &pg_connection).unwrap();
    assert!(inserted_user.id() > 0);
    assert_eq!(*inserted_user.uid(), uid);
    assert_eq!(inserted_user.vk_uid(), vk_uid);

    let selected_user = app_user::select_by_id(inserted_user.id(), &pg_connection);
    let selected_user = selected_user.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_user, selected_user);
}

#[test]
fn cant_insert_already_inserted_user() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let vk_uid = 2i32;
    delete_entry_with(&uid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let user_copy1 = app_user::new(uid, vk_uid);
    let user_copy2 = app_user::new(uid, vk_uid);

    app_user::insert(user_copy1, &pg_connection).unwrap();

    let second_insertion_result = app_user::insert(user_copy2, &pg_connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn cant_insert_user_with_already_used_uid() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
    let vk_uid1 = 3i32;
    let vk_uid2 = 4i32;
    delete_entry_with(&uid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let user1 = app_user::new(uid, vk_uid1);
    let user2 = app_user::new(uid, vk_uid2);

    app_user::insert(user1, &pg_connection).unwrap();

    let second_insertion_result = app_user::insert(user2, &pg_connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn cant_insert_user_with_already_used_vk_id() {
    let uid1 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440003").unwrap();
    let uid2 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440004").unwrap();
    let vk_uid = 5i32;
    delete_entry_with(&uid1);
    delete_entry_with(&uid2);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let user1 = app_user::new(uid1, vk_uid);
    let user2 = app_user::new(uid2, vk_uid);

    app_user::insert(user1, &pg_connection).unwrap();

    let second_insertion_result = app_user::insert(user2, &pg_connection);
    assert!(second_insertion_result.is_err());
}