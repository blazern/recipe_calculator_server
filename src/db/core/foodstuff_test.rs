extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::diesel_connection;
use db::core::foodstuff;
use db::core::testing_util;
use schema;
use testing_config;

const FOODSTUFF_NAME: &'static str = "foodstuff name for tests";
const FOODSTUFF_PROTEIN: f32 = 123456789_f32;
const FOODSTUFF_FATS: f32 = 123456789_f32;
const FOODSTUFF_CARBS: f32 = 123456789_f32;
const FOODSTUFF_CALORIES: f32 = 123456789_f32;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    testing_util_delete_entries_with!(
        app_user_uid,
        schema::foodstuff::table,
        schema::foodstuff::app_user_id);
}

#[test]
fn insertion_and_selection_work() {
    let app_user_foodstuff_id = 1;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440000").unwrap();

    delete_entries_with(&app_user_uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(inserted_foodstuff.id() > 0);
    assert_eq!(inserted_foodstuff.app_user_foodstuff_id(), app_user_foodstuff_id);
    assert_eq!(inserted_foodstuff.app_user_id(), app_user.id());

    let selected_foodstuff = foodstuff::select_by_id(inserted_foodstuff.id(), &connection);
    let selected_foodstuff = selected_foodstuff.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_foodstuff, selected_foodstuff);
}

#[test]
fn multiple_foodstuffs_can_depend_on_single_app_user() {
    let app_user_foodstuff_id1 = 2;
    let app_user_foodstuff_id2 = 3;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440001").unwrap();

    delete_entries_with(&app_user_uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_foodstuff1 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id1,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);
    let new_foodstuff2 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id2,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);

    foodstuff::insert(new_foodstuff1, &connection).unwrap();
    foodstuff::insert(new_foodstuff2, &connection).unwrap();
}

// 'aufi' stands for app_user_foodstuff_id
#[test]
fn multiple_foodstuffs_with_same_aufi_cannot_depend_on_single_app_user() {
    let app_user_foodstuff_id = 4;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440002").unwrap();

    delete_entries_with(&app_user_uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_foodstuff1 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);
    let new_foodstuff2 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);

    foodstuff::insert(new_foodstuff1, &connection).unwrap();
    let second_insertion_result = foodstuff::insert(new_foodstuff2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn can_make_foodstuff_unlisted() {
    let app_user_foodstuff_id = 5;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440003").unwrap();

    delete_entries_with(&app_user_uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true);

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(inserted_foodstuff.is_listed());
    let unlisted_foodstuff = foodstuff::unlist(inserted_foodstuff, &connection).unwrap();
    assert!(!unlisted_foodstuff.is_listed());
}

#[test]
fn making_already_unlisted_foodstuff_unlisted_does_nothing() {
    let app_user_foodstuff_id = 6;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440004").unwrap();

    delete_entries_with(&app_user_uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let app_user = app_user::insert(app_user::new(app_user_uid), &connection).unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        false);

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(!inserted_foodstuff.is_listed());
    let unlisted_foodstuff = foodstuff::unlist(inserted_foodstuff, &connection).unwrap();
    assert!(!unlisted_foodstuff.is_listed());
}