use diesel::Connection;
use std::str::FromStr;
use uuid::Uuid;

use crate::db;
use crate::db::core::app_user;
use crate::db::core::app_user::app_user as app_user_schema;
use crate::db::core::app_user::AppUser;
use crate::db::core::diesel_connection;
use crate::db::core::testing_util as dbtesting_utils;

#[test]
fn transition_rolls_back_progress_when_interrupted() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-006000000000").unwrap();

    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let connection = diesel_connection(&connection);

    delete_by_column!(
        app_user_schema::table,
        app_user_schema::uid,
        uid,
        connection
    )
    .unwrap();

    let invalid_id_value = -1;
    let mut id: i32 = invalid_id_value;
    let transaction_result = connection.transaction::<(), db::core::error::Error, _>(|| {
        let new_user = app_user::new(uid, "".to_string(), Uuid::new_v4());
        let user = insert!(AppUser, new_user, app_user_schema::table, connection);
        let user = user.unwrap();
        id = user.id();
        return Err("Failing transaction by test design".into());
    });
    assert!(transaction_result.is_err());
    assert_ne!(invalid_id_value, id);

    let user = select_by_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::id,
        id,
        connection
    );

    assert!(user.unwrap().is_none());
}

#[test]
fn operations_without_transactions_dont_roll_back() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-006000000001").unwrap();
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let connection = diesel_connection(&connection);

    delete_by_column!(
        app_user_schema::table,
        app_user_schema::uid,
        uid,
        connection
    )
    .unwrap();

    let new_user = app_user::new(uid, "".to_string(), Uuid::new_v4());
    let inserted_user = insert!(AppUser, new_user, app_user_schema::table, connection);
    assert!(inserted_user.is_ok());

    let selected_user = select_by_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::id,
        inserted_user.unwrap().id(),
        connection
    );

    assert!(selected_user.unwrap().is_some());
}
