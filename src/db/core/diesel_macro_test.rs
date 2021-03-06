use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::app_user::app_user as app_user_schema;
use crate::db::core::app_user::AppUser;
use crate::db::core::diesel_connection;
use crate::db::core::error::Error;
use crate::db::core::error::ErrorKind;
use crate::db::core::foodstuff;
use crate::db::core::foodstuff::foodstuff as foodstuff_schema;
use crate::db::core::foodstuff::Foodstuff;
use crate::db::core::testing_util as dbtesting_utils;

fn delete_user_by_uid(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    diesel::delete(app_user_schema::table.filter(app_user_schema::uid.eq(uid)))
        .execute(raw_connection)
        .unwrap();

    assert!(select_user_by_uid(uid).is_none());
}

fn select_user_by_uid(uid: &Uuid) -> Option<AppUser> {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    app_user_schema::table
        .filter(app_user_schema::uid.eq(uid))
        .first::<AppUser>(raw_connection)
        .optional()
        .unwrap()
}

fn create_foodstuff(
    app_user: &AppUser,
    app_user_foodstuff_id: i32,
    name: String,
) -> foodstuff::NewFoodstuff {
    foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        name,
        0i32,
        0i32,
        0i32,
        0i32,
        false,
    )
}

#[test]
fn insert_macro_works() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("00000000-0000-0000-0000-008000000000").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid, "".to_string(), Uuid::new_v4());
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());
}

#[test]
fn select_macro_works() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("00000000-0000-0000-0000-008000000001").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid, "".to_string(), Uuid::new_v4());
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    let result = select_by_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::uid,
        &uid,
        raw_connection
    );
    assert!(result.unwrap().is_some());
}

#[test]
fn delete_macro_works() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("00000000-0000-0000-0000-008000000002").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid, "".to_string(), Uuid::new_v4());
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    delete_by_column!(
        app_user_schema::table,
        app_user_schema::uid,
        &uid,
        raw_connection
    )
    .unwrap();
    assert!(select_user_by_uid(&uid).is_none());
}

#[test]
fn update_macro_works() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid1 = Uuid::from_str("00000000-0000-0000-0000-008000000003").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-008000000004").unwrap();
    delete_user_by_uid(&uid1);
    delete_user_by_uid(&uid2);
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_none());

    let inserted_user = insert!(
        AppUser,
        app_user::new(uid1, "".to_string(), Uuid::new_v4()),
        app_user_schema::table,
        raw_connection
    )
    .unwrap();
    assert!(select_user_by_uid(&uid1).is_some());
    assert!(select_user_by_uid(&uid2).is_none());

    update_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::id,
        inserted_user.id(),
        app_user_schema::uid,
        &uid2,
        raw_connection
    )
    .unwrap();
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_some());
}

#[test]
fn update_macro_returns_updated_values() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    // cleaning up
    let uid = Uuid::from_str("00000000-0000-0000-0000-008000000005").unwrap();
    let user = select_user_by_uid(&uid);
    match user {
        Some(user) => {
            diesel::delete(
                foodstuff_schema::table.filter(foodstuff_schema::app_user_id.eq(user.id())),
            )
            .execute(raw_connection)
            .unwrap();
        }
        _ => {}
    }
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let user = insert!(
        AppUser,
        app_user::new(uid, "".to_string(), Uuid::new_v4()),
        app_user_schema::table,
        raw_connection
    )
    .unwrap();

    let app_user_foodstuff_id1 = 1;
    let app_user_foodstuff_id2 = 2;
    let app_user_foodstuff_id3 = 3;
    let name1 = "name1";
    let name2 = "name2";
    // Note: second foodstuff uses same name as the first one
    let foodstuff1 = create_foodstuff(&user, app_user_foodstuff_id1, name1.to_string());
    let foodstuff2 = create_foodstuff(&user, app_user_foodstuff_id2, name1.to_string());
    let foodstuff3 = create_foodstuff(&user, app_user_foodstuff_id3, name2.to_string());

    let foodstuff1 = insert!(
        Foodstuff,
        foodstuff1,
        foodstuff_schema::table,
        raw_connection
    )
    .unwrap();
    let foodstuff2 = insert!(
        Foodstuff,
        foodstuff2,
        foodstuff_schema::table,
        raw_connection
    )
    .unwrap();
    let foodstuff3 = insert!(
        Foodstuff,
        foodstuff3,
        foodstuff_schema::table,
        raw_connection
    )
    .unwrap();

    assert_eq!(foodstuff1.name(), foodstuff2.name());
    assert_ne!(foodstuff1.name(), foodstuff3.name());

    let new_name = "new-name";
    let updated_foodstuffs = update_column!(
        Foodstuff,
        foodstuff_schema::table,
        foodstuff_schema::name,
        &name1,
        foodstuff_schema::name,
        &new_name,
        raw_connection
    )
    .unwrap();

    assert_eq!(2, updated_foodstuffs.len());
    assert!(updated_foodstuffs
        .iter()
        .all(|foodstuff| foodstuff.name() == new_name));
    assert!(updated_foodstuffs
        .iter()
        .any(|foodstuff| foodstuff.id() == foodstuff1.id()));
    assert!(updated_foodstuffs
        .iter()
        .any(|foodstuff| foodstuff.id() == foodstuff2.id()));
}

#[test]
fn unique_violation_error_returned_on_unique_violation() {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("00000000-0000-0000-0000-008000000006").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user1 = app_user::new(uid, "name1".to_string(), Uuid::new_v4());
    let new_user2 = app_user::new(uid, "name2".to_string(), Uuid::new_v4());
    insert!(AppUser, new_user1, app_user_schema::table, raw_connection).unwrap();

    let second_insertion_result =
        insert!(AppUser, new_user2, app_user_schema::table, raw_connection);
    match second_insertion_result {
        Ok(_) => {
            panic!("Insertion with uid-duplicate expected to fail");
        }
        Err(error) => {
            match &error {
                &Error(ErrorKind::UniqueViolation(_), _) => {
                    // Ok
                }
                _ => {
                    panic!("Unexpected error: {:?}", error);
                }
            }
        }
    }
}
