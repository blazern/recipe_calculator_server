extern crate diesel;
extern crate uuid;

use diesel::Connection;
use std::str::FromStr;
use uuid::Uuid;

use db;
use db::core::app_user;
use db::core::app_user::AppUser;
use db::core::connection::DBConnection;
use db::core::diesel_connection;
use schema;

#[test]
fn transition_rolls_back_progress_when_interrupted() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-646655440001").unwrap();

    let connection = DBConnection::for_admin_user().unwrap();
    let connection = diesel_connection(&connection);

    delete_by_column!(
        schema::app_user::table, schema::app_user::uid, uid, connection).unwrap();

    let invalid_id_value = -1;
    let mut id: i32 = invalid_id_value;
    let transaction_result = connection.transaction::<(), db::core::error::Error, _>(|| {
        let new_user = app_user::new(uid);
        let user = insert!(AppUser, new_user, schema::app_user::table, connection);
        let user = user.unwrap();
        id = user.id();
        Err("Failing transaction by test design")?
    });
    assert!(transaction_result.is_err());
    assert_ne!(invalid_id_value, id);

    let user =
        select_by_column!(AppUser, schema::app_user::table, schema::app_user::id, id, connection);

    assert!(user.unwrap().is_none());
}

#[test]
fn operations_without_transactions_dont_roll_back() {
    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-646655440002").unwrap();
    let connection = DBConnection::for_admin_user().unwrap();
    let connection = diesel_connection(&connection);

    delete_by_column!(
        schema::app_user::table, schema::app_user::uid, uid, connection).unwrap();

    let new_user = app_user::new(uid);
    let inserted_user = insert!(AppUser, new_user, schema::app_user::table, connection);
    assert!(inserted_user.is_ok());

    let selected_user =
        select_by_column!(
            AppUser,
            schema::app_user::table,
            schema::app_user::id,
            inserted_user.unwrap().id(),
            connection);

    assert!(selected_user.unwrap().is_some());
}