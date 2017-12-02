extern crate diesel;
extern crate uuid;

use std::env;
use std::str::FromStr;
use diesel::Connection;
use diesel::pg::PgConnection;
use uuid::Uuid;

use db::app_user;
use schema;

include!("../testing_config.rs.inc");
include!("psql_admin_url.rs.inc");

// Cleaning up before tests
fn delete_entry_with(uid: &Uuid) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let pg_connection = PgConnection::establish(&psql_admin_url).unwrap();
    delete_by_column!(schema::app_user::table, schema::app_user::uid, uid, &pg_connection)
        .expect("Deletion shouldn't fail");
    let deleted_user =
        select_by_column!(app_user::AppUser, schema::app_user::table, schema::app_user::uid, uid, &pg_connection);
    assert!(deleted_user.expect("Selection shouldn't fail").is_none());
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    delete_entry_with(&uid);

    let config = get_testing_config();
    let new_user = app_user::new(uid);
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let inserted_user = app_user::insert(new_user, &pg_connection).unwrap();
    assert!(inserted_user.id() > 0);
    assert_eq!(*inserted_user.uid(), uid);

    let selected_user = app_user::select_by_id(inserted_user.id(), &pg_connection);
    let selected_user = selected_user.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_user, selected_user);
}

#[test]
fn cant_insert_user_with_already_used_uid() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
    delete_entry_with(&uid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    let user1 = app_user::new(uid);
    let user2 = app_user::new(uid);

    app_user::insert(user1, &pg_connection).unwrap();

    let second_insertion_result = app_user::insert(user2, &pg_connection);
    assert!(second_insertion_result.is_err());
}
