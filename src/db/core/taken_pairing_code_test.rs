extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::diesel_connection;
use db::core::taken_pairing_code;
use db::core::testing_util as dbtesting_utils;
use db::core::util::delete_app_user;

// Cleaning up before tests
fn delete_ranges_with_family(family: &str) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    taken_pairing_code::delete_family(family, &connection).unwrap();
}

fn delete_user_with_uid(uid: &Uuid) {
    use super::taken_pairing_code as taken_pairing_code_schema;
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let user = app_user::select_by_uid(&uid, &connection).unwrap();
    match user {
        Some(user) => {
            delete_by_column!(
                taken_pairing_code_schema::table,
                taken_pairing_code_schema::app_user_id,
                user.id(),
                diesel_connection(&connection)
            ).unwrap();
        },
        None => {}
    };
    delete_app_user(&uid, &connection).unwrap();
}

#[test]
fn insertion_and_selection_work() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002220000000").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user = app_user::insert(app_user::new(uid, "".to_owned(), Uuid::new_v4()), &conn);
    let user = user.unwrap();

    let code = taken_pairing_code::new(&user, 10, 100, fam.to_owned());
    let inserted_code = taken_pairing_code::insert(code, &conn).unwrap();
    assert!(inserted_code.id() > 0);
    assert_eq!(fam, inserted_code.family());
    assert_eq!(10, inserted_code.val());
    assert_eq!(100, inserted_code.creation_time());

    let selected_code1 = taken_pairing_code::select_by_id(inserted_code.id(), &conn);
    let selected_code1 = selected_code1.unwrap().unwrap(); // unwrapping Result and Option
    let selected_code2 = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn);
    let selected_code2 = selected_code2.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_code, selected_code1);
    assert_eq!(inserted_code, selected_code2);
}

#[test]
fn delete_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002220000001").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002220000002").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002220000003").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 = app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user3 = app_user::insert(app_user::new(uid3, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let code1 = taken_pairing_code::new(&user1, 10, 100, fam.to_owned());
    let code1 = taken_pairing_code::insert(code1, &conn).unwrap();
    let code2 = taken_pairing_code::new(&user2, 11, 101, fam.to_owned());
    let code2 = taken_pairing_code::insert(code2, &conn).unwrap();
    let code3 = taken_pairing_code::new(&user3, 12, 102, "another fam".to_owned());
    let code3 = taken_pairing_code::insert(code3, &conn).unwrap();

    assert!(taken_pairing_code::select_by_id(code1.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code2.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code3.id(), &conn).unwrap().is_some());
    taken_pairing_code::delete_family(&fam, &conn).unwrap();
    assert!(taken_pairing_code::select_by_id(code1.id(), &conn).unwrap().is_none());
    assert!(taken_pairing_code::select_by_id(code2.id(), &conn).unwrap().is_none());
    assert!(taken_pairing_code::select_by_id(code3.id(), &conn).unwrap().is_some());
}

#[test]
fn delete_old_ranges() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002220000004").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002220000005").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002220000006").unwrap();
    let uid4 = Uuid::from_str("00000000-0000-0000-0000-002220000007").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    delete_user_with_uid(&uid4);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 = app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user3 = app_user::insert(app_user::new(uid3, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user4 = app_user::insert(app_user::new(uid4, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let code1 = taken_pairing_code::new(&user1, 10, 100, fam.to_owned());
    let code1 = taken_pairing_code::insert(code1, &conn).unwrap();
    let code2 = taken_pairing_code::new(&user2, 11, 101, fam.to_owned());
    let code2 = taken_pairing_code::insert(code2, &conn).unwrap();
    let code3 = taken_pairing_code::new(&user3, 12, 102, fam.to_owned());
    let code3 = taken_pairing_code::insert(code3, &conn).unwrap();
    let code4 = taken_pairing_code::new(&user4, 10, 100, "other fam".to_owned());
    let code4 = taken_pairing_code::insert(code4, &conn).unwrap();

    assert!(taken_pairing_code::select_by_id(code1.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code2.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code3.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code4.id(), &conn).unwrap().is_some());
    let deleted = taken_pairing_code::delete_older_than(102, &fam, &conn).unwrap();
    assert!(taken_pairing_code::select_by_id(code1.id(), &conn).unwrap().is_none());
    assert!(taken_pairing_code::select_by_id(code2.id(), &conn).unwrap().is_none());
    assert!(taken_pairing_code::select_by_id(code3.id(), &conn).unwrap().is_some());
    assert!(taken_pairing_code::select_by_id(code4.id(), &conn).unwrap().is_some());

    assert_eq!(2, deleted.len());
    assert_eq!(code1, deleted[0]);
    assert_eq!(code2, deleted[1]);
}

#[test]
fn multiple_codes_cannot_depend_on_single_app_user_in_same_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002220000008").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user = app_user::insert(app_user::new(uid, "".to_owned(), Uuid::new_v4()), &conn);
    let user = user.unwrap();

    let code1 = taken_pairing_code::new(&user, 10, 100, fam.to_owned());
    let code2 = taken_pairing_code::new(&user, 11, 101, fam.to_owned());
    assert!(taken_pairing_code::insert(code1, &conn).is_ok());
    assert!(taken_pairing_code::insert(code2, &conn).is_err());
}

#[test]
fn multiple_codes_can_depend_on_single_app_user_in_different_families() {
    let fam1 = format!("{}{}", file!(), line!());
    let fam2 = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam1);
    delete_ranges_with_family(&fam2);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002220000009").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user = app_user::insert(app_user::new(uid, "".to_owned(), Uuid::new_v4()), &conn);
    let user = user.unwrap();

    let code1 = taken_pairing_code::new(&user, 10, 100, fam1.to_owned());
    let code2 = taken_pairing_code::new(&user, 11, 101, fam2.to_owned());
    assert!(taken_pairing_code::insert(code1, &conn).is_ok());
    assert!(taken_pairing_code::insert(code2, &conn).is_ok());
}

#[test]
fn multiple_codes_cannot_have_same_val_in_same_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002220000010").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002220000011").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn);
    let user1 = user1.unwrap();
    let user2 = app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn);
    let user2 = user2.unwrap();

    let code1 = taken_pairing_code::new(&user1, 10, 100, fam.to_owned());
    let code2 = taken_pairing_code::new(&user2, 10, 101, fam.to_owned());
    assert!(taken_pairing_code::insert(code1, &conn).is_ok());
    assert!(taken_pairing_code::insert(code2, &conn).is_err());
}

#[test]
fn multiple_codes_can_have_same_val_in_different_families() {
    let fam1 = format!("{}{}", file!(), line!());
    let fam2 = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam1);
    delete_ranges_with_family(&fam2);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002220000012").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002220000013").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn);
    let user1 = user1.unwrap();
    let user2 = app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn);
    let user2 = user2.unwrap();

    let code1 = taken_pairing_code::new(&user1, 10, 100, fam1.to_owned());
    let code2 = taken_pairing_code::new(&user2, 10, 101, fam2.to_owned());
    assert!(taken_pairing_code::insert(code1, &conn).is_ok());
    assert!(taken_pairing_code::insert(code2, &conn).is_ok());
}
