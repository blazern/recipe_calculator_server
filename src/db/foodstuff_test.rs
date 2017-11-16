extern crate diesel;
extern crate uuid;

use diesel::Connection;
use diesel::ExecuteDsl;
use diesel::ExpressionMethods;
use diesel::FilterDsl;
use diesel::pg::PgConnection;
use std::env;
use std::str::FromStr;
use uuid::Uuid;

use db::app_user;
use db::foodstuff;
use schema;

include!("../testing_config.rs.inc");
include!("psql_admin_url.rs.inc");

const FOODSTUFF_NAME: &'static str = "foodstuff name for tests";
const FOODSTUFF_PROTEIN: f32 = 123456789_f32;
const FOODSTUFF_FATS: f32 = 123456789_f32;
const FOODSTUFF_CARBS: f32 = 123456789_f32;
const FOODSTUFF_CALORIES: f32 = 123456789_f32;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    let app_user =
        select_by_column!(
            app_user::AppUser,
            schema::app_user::table,
            schema::app_user::uid,
            app_user_uid,
            &connection);

    let app_user = app_user.unwrap();
    if app_user.is_none() {
        // AppUser already deleted - foodstuffs are connected to it by foreign key, so they are
        // deleted too by now, because otherwise DB wouldn't let us delete
        return;
    }
    let app_user = app_user.unwrap();

    delete_by_column!(
        schema::foodstuff::table,
        schema::foodstuff::app_user_id,
        app_user.id(),
        &connection).unwrap();

    delete_by_column!(
        schema::app_user::table,
        schema::app_user::id,
        app_user.id(),
        &connection).unwrap();
}

#[test]
fn insertion_and_selection_work() {
    let app_user_foodstuff_id = 1;
    let app_user_uid = Uuid::from_str("550e8400-e29b-f00d-a716-a46655440000").unwrap();

    delete_entries_with(&app_user_uid);

    let config = get_testing_config();
    let connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

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

    let config = get_testing_config();
    let connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

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

    let config = get_testing_config();
    let connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

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

    let config = get_testing_config();
    let connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

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

    let config = get_testing_config();
    let connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

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