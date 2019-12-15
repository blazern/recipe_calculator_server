extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::device;
use db::core::testing_util as dbtesting_utils;

// Cleaning up before tests
fn delete_entries_with(app_user_uid: &Uuid) {
    use db::core::util::delete_app_user;
    delete_app_user(
        &app_user_uid,
        &dbtesting_utils::testing_connection_for_server_user().unwrap()).unwrap();
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let uuid = Uuid::from_str("00000000-0000-0000-0000-007000000000").unwrap();
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-007000000001").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user =
        app_user::insert(
            app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
            &connection).unwrap();

    let new_device = device::new(uuid, &app_user);

    let inserted_device = device::insert(new_device, &connection).unwrap();
    assert!(inserted_device.id() > 0);
    assert_eq!(*inserted_device.uuid(), uuid);
    assert_eq!(app_user.id(), inserted_device.app_user_id());

    let selected_device = device::select_by_id(inserted_device.id(), &connection);
    let selected_device = selected_device.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_device, selected_device);
}

#[test]
fn cant_insert_device_with_already_used_uuid() {
    let uuid = Uuid::from_str("00000000-0000-0000-0000-007000000002").unwrap();
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-007000000003").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user =
        app_user::insert(
            app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
            &connection).unwrap();

    let device_copy1 = device::new(uuid, &app_user);
    let device_copy2 = device::new(uuid, &app_user);

    device::insert(device_copy1, &connection).unwrap();

    let second_insertion_result = device::insert(device_copy2, &connection);
    assert!(second_insertion_result.is_err());
}

#[test]
fn multiple_devices_can_depend_on_single_app_user() {
    let uuid1 = Uuid::from_str("00000000-0000-0000-0000-007000000004").unwrap();
    let uuid2 = Uuid::from_str("00000000-0000-0000-0000-007000000005").unwrap();
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-007000000006").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user =
        app_user::insert(
            app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
            &connection).unwrap();

    let device1 = device::new(uuid1, &app_user);
    let device2 = device::new(uuid2, &app_user);

    device::insert(device1, &connection).unwrap();
    device::insert(device2, &connection).unwrap();
}

#[test]
fn can_select_by_uuid() {
    let uuid = Uuid::from_str("00000000-0000-0000-0000-007000000007").unwrap();
    let app_user_uid = Uuid::from_str("00000000-0000-0000-0000-007000000008").unwrap();
    delete_entries_with(&app_user_uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let app_user =
        app_user::insert(
            app_user::new(app_user_uid, "".to_string(), Uuid::new_v4()),
            &connection).unwrap();

    let inserted_device = device::insert(device::new(uuid.clone(), &app_user), &connection).unwrap();
    let selected_device = device::select_by_uuid(&uuid, &connection).unwrap().unwrap();

    assert_eq!(inserted_device, selected_device);
}

#[test]
fn can_delete_device_by_id() {
    let uuid = Uuid::from_str("00000000-0000-0000-0000-007000000009").unwrap();
    let uid = Uuid::from_str("00000000-0000-0000-0000-007000000010").unwrap();
    delete_entries_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let inserted_user =
        app_user::insert(
            app_user::new(uid, "".to_string(), Uuid::new_v4()),
            &connection).unwrap();
    let inserted_device = device::insert(device::new(uuid, &inserted_user), &connection).unwrap();

    device::delete_by_id(inserted_device.id(), &connection).unwrap();
    let deleted_device = device::select_by_id(inserted_device.id(), &connection).unwrap();

    assert!(deleted_device.is_none());
}

#[test]
fn cant_delete_device_with_client_connection() {
    let uuid = Uuid::from_str("00000000-0000-0000-0000-007000000011").unwrap();
    let uid = Uuid::from_str("00000000-0000-0000-0000-007000000012").unwrap();
    delete_entries_with(&uid);

    let pg_client_connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted_user =
        app_user::insert(
            app_user::new(uid, "".to_string(), Uuid::new_v4()),
            &pg_client_connection).unwrap();
    let inserted_device = device::insert(device::new(uuid, &inserted_user), &pg_client_connection).unwrap();

    let device_deletion_result = device::delete_by_id(inserted_device.id(), &pg_client_connection);

    assert!(device_deletion_result.is_err());
}
