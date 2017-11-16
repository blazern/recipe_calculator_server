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
use db::vk_user;
use schema;
use error;

include!("../testing_config.rs.inc");
include!("psql_admin_url.rs.inc");

// Cleaning up before tests
fn delete_entries_with(vk_uids: &[i32]) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let pg_connection = PgConnection::establish(&psql_admin_url).unwrap();

    pg_connection.transaction::<_, error::Error, _>(|| {
        // Memorize existing entries
        let mut existing_entries = Vec::new();
        for vk_uid in vk_uids {
            let selected_entry = select_entry_with(*vk_uid, &pg_connection);
            match selected_entry {
                Some(entry) => {
                    existing_entries.push(entry);
                }
                _ => {}
            }
        }

        // Delete them
        for vk_uid in vk_uids {
            delete_by_column!(
                    schema::vk_user::table,
                    schema::vk_user::vk_uid,
                    *vk_uid,
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


        for vk_uid in vk_uids {
            let deleted_entry = select_entry_with(*vk_uid, &pg_connection);
            assert!(deleted_entry.is_none());
        }

        Ok(())
    }).unwrap();
}

fn delete_entry_with(vk_uid: i32) {
    delete_entries_with(&[vk_uid])
}

fn select_entry_with(vk_uid: i32, pg_connection: &PgConnection) -> Option<vk_user::VkUser> {
    return select_by_column!(
        vk_user::VkUser,
        schema::vk_user::table,
        schema::vk_user::vk_uid,
        vk_uid,
        pg_connection).unwrap();
}

// NOTE: different UUIDs and VK IDs must be used in each tests, because tests are run in parallel
// and usage of same IDs would cause race conditions.

#[test]
fn insertion_and_selection_work() {
    let vk_uid = 1;
    delete_entry_with(vk_uid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655540000")?;
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection)?;

        let new_vk_user = vk_user::new(vk_uid, &app_user);

        let inserted_vk_user = vk_user::insert(new_vk_user, &pg_connection)?;
        assert!(inserted_vk_user.id() > 0);
        assert_eq!(inserted_vk_user.vk_uid(), vk_uid);
        assert_eq!(app_user.id(), inserted_vk_user.app_user_id());

        let selected_vk_user = vk_user::select_by_id(inserted_vk_user.id(), &pg_connection);
        let selected_vk_user = selected_vk_user.unwrap().unwrap(); // unwrapping Result and Option
        assert_eq!(inserted_vk_user, selected_vk_user);

        Ok(())
    }).unwrap();
}

#[test]
fn cant_insert_vk_user_with_already_used_vk_uid() {
    let vk_uid = 2;
    delete_entry_with(vk_uid);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655450001").unwrap();
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection).unwrap();

        let vk_user_copy1 = vk_user::new(vk_uid, &app_user);
        let vk_user_copy2 = vk_user::new(vk_uid, &app_user);

        vk_user::insert(vk_user_copy1, &pg_connection).unwrap();

        let second_insertion_result = vk_user::insert(vk_user_copy2, &pg_connection);
        assert!(second_insertion_result.is_err());

        Ok(())
    }).unwrap();
}

#[test]
fn multiple_vk_users_cannot_depend_on_single_app_user() {
    let vk_uid1 = 3;
    let vk_uid2 = 4;
    delete_entries_with(&[vk_uid1, vk_uid2]);

    let config = get_testing_config();
    let pg_connection = PgConnection::establish(config.psql_diesel_url_client_user()).unwrap();

    // Need to revert insertions if test fails
    pg_connection.transaction::<_, error::Error, _>(|| {
        let app_user_uid = Uuid::from_str("550e8400-e29b-41d4-a716-a46655450002").unwrap();
        let app_user = app_user::insert(app_user::new(app_user_uid), &pg_connection).unwrap();

        let vk_user1 = vk_user::new(vk_uid1, &app_user);
        let vk_user2 = vk_user::new(vk_uid2, &app_user);

        vk_user::insert(vk_user1, &pg_connection).unwrap();

        let second_user_selection_result = vk_user::insert(vk_user2, &pg_connection);
        assert!(second_user_selection_result.is_err());

        Ok(())
    }).unwrap();
}