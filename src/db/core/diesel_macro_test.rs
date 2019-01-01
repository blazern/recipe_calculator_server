extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use diesel::ExecuteDsl;
use diesel::ExpressionMethods;
use diesel::FilterDsl;
use diesel::OptionalExtension;
use uuid::Uuid;

use db::core::app_user;
use db::core::app_user::AppUser;
use db::core::connection::DBConnection;
use db::core::diesel_connection;
use db::core::error::Error;
use db::core::error::ErrorKind;
use db::core::foodstuff;
use db::core::foodstuff::Foodstuff;
use db::core::app_user::app_user as app_user_schema;
use db::core::foodstuff::foodstuff as foodstuff_schema;

fn delete_user_by_uid(uid: &Uuid) {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    diesel::delete(
        app_user_schema::table.filter(
            app_user_schema::uid.eq(uid)))
        .execute(raw_connection).unwrap();

    assert!(select_user_by_uid(uid).is_none());
}

fn select_user_by_uid(uid: &Uuid) -> Option<AppUser> {
    use diesel::FirstDsl;

    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    return app_user_schema::table.filter(app_user_schema::uid.eq(uid))
        .first::<AppUser>(raw_connection).optional().unwrap();
}

fn create_foodstuff(app_user: &AppUser, app_user_foodstuff_id: i32, name: String)
        -> foodstuff::NewFoodstuff {
    return foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        name,
        0i32,
        0i32,
        0i32,
        0i32,
        false);
}

#[test]
fn insert_macro_works() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440000").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());
}

#[test]
fn select_macro_works() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440001").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    let result = select_by_column!(AppUser, app_user_schema::table, app_user_schema::uid, &uid, raw_connection);
    assert!(result.unwrap().is_some());
}

#[test]
fn delete_macro_works() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440002").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    delete_by_column!(app_user_schema::table, app_user_schema::uid, &uid, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid).is_none());
}

#[test]
fn update_macro_works() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid1 = Uuid::from_str("550a8400-e29b-41d4-a716-446655440003").unwrap();
    let uid2 = Uuid::from_str("550a8400-e29b-41d4-a716-446655440004").unwrap();
    delete_user_by_uid(&uid1);
    delete_user_by_uid(&uid2);
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_none());

    let inserted_user = insert!(AppUser, app_user::new(uid1), app_user_schema::table, raw_connection).unwrap();
    assert!(select_user_by_uid(&uid1).is_some());
    assert!(select_user_by_uid(&uid2).is_none());

    update_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::id,
        inserted_user.id(),
        app_user_schema::uid,
        &uid2,
        raw_connection).unwrap();
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_some());
}

#[test]
fn update_macro_returns_updated_values() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    // cleaning up
    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440005").unwrap();
    let user = select_user_by_uid(&uid);
    match user {
        Some(user) => {
            diesel::delete(
                foodstuff_schema::table.filter(
                    foodstuff_schema::app_user_id.eq(user.id())))
                .execute(raw_connection).unwrap();
        }
        _ => {}
    }
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let user = insert!(AppUser, app_user::new(uid), app_user_schema::table, raw_connection).unwrap();

    let app_user_foodstuff_id1 = 1;
    let app_user_foodstuff_id2 = 2;
    let app_user_foodstuff_id3 = 3;
    let name1 = "name1";
    let name2 = "name2";
    // Note: second foodstuff uses same name as the first one
    let foodstuff1 =
        create_foodstuff(&user, app_user_foodstuff_id1, name1.to_string());
    let foodstuff2 =
        create_foodstuff(&user, app_user_foodstuff_id2, name1.to_string());
    let foodstuff3 =
        create_foodstuff(&user, app_user_foodstuff_id3, name2.to_string());

    let foodstuff1 = insert!(Foodstuff, foodstuff1, foodstuff_schema::table, raw_connection).unwrap();
    let foodstuff2 = insert!(Foodstuff, foodstuff2, foodstuff_schema::table, raw_connection).unwrap();
    let foodstuff3 = insert!(Foodstuff, foodstuff3, foodstuff_schema::table, raw_connection).unwrap();

    assert_eq!(foodstuff1.name(), foodstuff2.name());
    assert_ne!(foodstuff1.name(), foodstuff3.name());

    let new_name = "new-name";
    let updated_foodstuffs =
        update_column!(
            Foodstuff,
            foodstuff_schema::table,
            foodstuff_schema::name,
            &name1,
            foodstuff_schema::name,
            &new_name,
            raw_connection).unwrap();

    assert_eq!(2, updated_foodstuffs.len());
    assert!(updated_foodstuffs.iter().all(|foodstuff| foodstuff.name() == new_name));
    assert!(updated_foodstuffs.iter().any(|foodstuff| foodstuff.id() == foodstuff1.id()));
    assert!(updated_foodstuffs.iter().any(|foodstuff| foodstuff.id() == foodstuff2.id()));
}

#[test]
fn unique_violation_error_returned_on_unique_violation() {
    let connection = DBConnection::for_admin_user().unwrap();
    let raw_connection = diesel_connection(&connection);

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440006").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user1 = app_user::new(uid);
    let new_user2 = app_user::new(uid);
    insert!(AppUser, new_user1, app_user_schema::table, raw_connection).unwrap();

    let second_insertion_result = insert!(AppUser, new_user2, app_user_schema::table, raw_connection);
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