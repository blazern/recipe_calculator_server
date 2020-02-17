use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::foodstuff;
use crate::db::core::testing_util as dbtesting_utils;

const FOODSTUFF_NAME: &str = "foodstuff name for tests";
const FOODSTUFF_PROTEIN: i32 = 123_456_789_i32;
const FOODSTUFF_FATS: i32 = 123_456_789_i32;
const FOODSTUFF_CARBS: i32 = 123_456_789_i32;
const FOODSTUFF_CALORIES: i32 = 123_456_789_i32;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    use crate::db::core::util::delete_app_user;
    delete_app_user(
        &app_user_uid,
        &dbtesting_utils::testing_connection_for_server_user().unwrap(),
    )
    .unwrap();
}

#[test]
fn insertion_and_selection_work() {
    let app_user_foodstuff_id = 1;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-005000000000").unwrap();

    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(inserted_foodstuff.id() > 0);
    assert_eq!(
        inserted_foodstuff.app_user_foodstuff_id(),
        app_user_foodstuff_id
    );
    assert_eq!(inserted_foodstuff.app_user_id(), app_user.id());

    let selected_foodstuff = foodstuff::select_by_id(inserted_foodstuff.id(), &connection);
    let selected_foodstuff = selected_foodstuff.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_foodstuff, selected_foodstuff);
}

#[test]
fn multiple_foodstuffs_can_depend_on_single_app_user() {
    let app_user_foodstuff_id1 = 2;
    let app_user_foodstuff_id2 = 3;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-005000000001").unwrap();

    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_foodstuff1 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id1,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );
    let new_foodstuff2 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id2,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );

    foodstuff::insert(new_foodstuff1, &connection).unwrap();
    foodstuff::insert(new_foodstuff2, &connection).unwrap();
}

// 'aufi' stands for app_user_foodstuff_id
#[test]
fn multiple_foodstuffs_with_same_aufi_cannot_depend_on_single_app_user() {
    let app_user_foodstuff_id = 4;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-005000000002").unwrap();

    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_foodstuff1 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );
    let new_foodstuff2 = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );

    foodstuff::insert(new_foodstuff1, &connection).unwrap();
    let second_insertion_result = foodstuff::insert(new_foodstuff2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn can_make_foodstuff_unlisted() {
    let app_user_foodstuff_id = 5;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-005000000003").unwrap();

    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        true,
    );

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(inserted_foodstuff.is_listed());
    let unlisted_foodstuff = foodstuff::unlist(inserted_foodstuff, &connection)
        .unwrap()
        .unwrap();
    assert!(!unlisted_foodstuff.is_listed());
}

#[test]
fn making_already_unlisted_foodstuff_unlisted_does_nothing() {
    let app_user_foodstuff_id = 6;
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-005000000004").unwrap();

    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user = app_user::insert(
        app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
        &connection,
    )
    .unwrap();

    let new_foodstuff = foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        FOODSTUFF_NAME.to_string(),
        FOODSTUFF_PROTEIN,
        FOODSTUFF_FATS,
        FOODSTUFF_CARBS,
        FOODSTUFF_CALORIES,
        false,
    );

    let inserted_foodstuff = foodstuff::insert(new_foodstuff, &connection).unwrap();
    assert!(!inserted_foodstuff.is_listed());
    let unlisted_foodstuff = foodstuff::unlist(inserted_foodstuff, &connection)
        .unwrap()
        .unwrap();
    assert!(!unlisted_foodstuff.is_listed());
}
