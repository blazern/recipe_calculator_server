extern crate diesel;
extern crate uuid;

use std::env;
use std::str::FromStr;
use diesel::Connection;
use diesel::ExecuteDsl;
use diesel::ExpressionMethods;
use diesel::FilterDsl;
use diesel::pg::PgConnection;
use uuid::Uuid;

use db::app_user;
use db::device;
use schema;
use error;

include!("../testing_config.rs.inc");
include!("psql_admin_url.rs.inc");

// Cleaning up before tests
fn delete_entries_with(uuids: &[&Uuid]) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let pg_connection = PgConnection::establish(&psql_admin_url).unwrap();

    pg_connection.transaction::<_, error::Error, _>(|| {
        // Memorize existing entries
        let mut existing_entries = Vec::new();
        for uuid in uuids {
            let selected_entry = select_entry_with(&uuid, &pg_connection);
            match selected_entry {
                Some(entry) => {
                    existing_entries.push(entry);
                }
                _ => {}
            }
        }

        // Delete them
        for uuid in uuids {
            delete_by_column!(
                    schema::device::table,
                    schema::device::uuid,
                    *uuid,
                    &pg_connection)?;
        }

        // Delete their AppUsers (so that foreign key constraint wouldn't stop us).
        for entry in existing_entries {
            delete_by_column!(
                    schema::app_user::table,
                    schema::app_user::id,
                    entry.app_user_id(),
                    &pg_connection)?;
        }


        for uuid in uuids {
            let deleted_entry = select_entry_with(&uuid, &pg_connection);
            assert!(deleted_entry.is_none());
        }

        Ok(())
    }).unwrap();
}

fn delete_entry_with(uuid: &Uuid) {
    delete_entries_with(&[uuid])
}

fn select_entry_with(uuid: &Uuid, pg_connection: &PgConnection) -> Option<device::Device> {
    return select_by_column!(
        device::Device,
        schema::device::table,
        schema::device::uuid,
        uuid,
        pg_connection).unwrap();
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let uuid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    delete_entry_with(&uuid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655440000")?;
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection)?;

        let new_device = device::new(uuid, &app_user);

        let inserted_device = device::insert(new_device, &pg_connection)?;
        assert!(inserted_device.id() > 0);
        assert_eq!(*inserted_device.uuid(), uuid);
        assert_eq!(app_user.id(), inserted_device.app_user_id());

        let selected_device = device::select_by_id(inserted_device.id(), &pg_connection);
        let selected_device = selected_device.unwrap().unwrap(); // unwrapping Result and Option
        assert_eq!(inserted_device, selected_device);

        Ok(())
    }).unwrap();
}

#[test]
fn cant_insert_device_with_already_used_uuid() {
    let uuid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    delete_entry_with(&uuid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655440001").unwrap();
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection).unwrap();

        let device_copy1 = device::new(uuid, &app_user);
        let device_copy2 = device::new(uuid, &app_user);

        device::insert(device_copy1, &pg_connection).unwrap();

        let second_insertion_result = device::insert(device_copy2, &pg_connection);
        assert!(second_insertion_result.is_err());

        Ok(())
    }).unwrap();
}

#[test]
fn multiple_devices_can_depend_on_single_app_user() {
    let uuid1 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
    let uuid2 = Uuid::from_str("550e8400-e29b-41d4-a716-446655440003").unwrap();
    delete_entries_with(&[&uuid1, &uuid2]);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655440002").unwrap();
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection).unwrap();

        let device1 = device::new(uuid1, &app_user);
        let device2 = device::new(uuid2, &app_user);

        device::insert(device1, &pg_connection).unwrap();
        device::insert(device2, &pg_connection).unwrap();

        Ok(())
    }).unwrap();
}