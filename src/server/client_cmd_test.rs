extern crate diesel;
extern crate uuid;

use std::str::FromStr;

use uuid::Uuid;

use server::error::Error;
use server::error::ErrorKind::UniqueUuidCreationError;
use db::core::app_user;
use db::core::testing_util as dbtesting_utils;
use server::client_cmd;

struct SameUuidGenerator {
    uuid: Uuid,
}

impl SameUuidGenerator {
    fn new(uid: Uuid) -> SameUuidGenerator {
        SameUuidGenerator{ uuid: uid }
    }
}

impl client_cmd::UuidGenerator for SameUuidGenerator {
    fn generate(&mut self) -> Uuid {
        return self.uuid.clone();
    }
}

// Cleaning up before tests
fn delete_app_user_with(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let selected_app_user = app_user::select_by_uid(&uid, &connection);
    match selected_app_user {
        Ok(Some(app_user)) => {
            app_user::delete_by_id(app_user.id(), &connection).unwrap();
        }
        _ => {}
    }
}

#[test]
fn user_registration_works() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000000").unwrap();
    delete_app_user_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let mut uid_generator = SameUuidGenerator::new(uid.clone());
    client_cmd::register_user_by_uuid_generator("name", "type", "token", &mut uid_generator, &connection).unwrap();

    // making sure app_user is inserted
    let user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    assert_eq!("name", user.name())
}

#[test]
fn user_registration_returns_duplication_error_on_uid_duplication() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000005").unwrap();
    delete_app_user_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let mut uid_generator = SameUuidGenerator::new(uid.clone());
    client_cmd::register_user_by_uuid_generator("name1", "type1", "token1", &mut uid_generator, &connection).unwrap();

    let second_registration_result =
        client_cmd::register_user_by_uuid_generator("name2", "type2", "token2", &mut uid_generator, &connection);
    match second_registration_result {
        Err(Error(UniqueUuidCreationError(_), _)) => {
            // OK
        }
        Err(err) => {
            panic!("Expected error::UniqueUuidCreationError, but got another error: {:?}", err);
        }
        Ok(_) => {
            panic!("Expected error::UniqueUuidCreationError, but registration was successful!");
        }
    }
}

#[test]
fn uid_duplication_leaves_vk_user_not_created() {
    // TODO
//    let uid1 = Uuid::from_str("00000000-0000-0000-0000-001000000008").unwrap();
//    let uid2 = Uuid::from_str("00000000-0000-0000-0000-001000000009").unwrap();
//    let device_uuid = Uuid::from_str("00000000-0000-0000-0000-001000000010").unwrap();
//    delete_device_with(&device_uuid);
//    delete_app_user_with(&uid1);
//    delete_app_user_with(&uid2);
//
//    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();
//
//    let mut uid_generator1 = SameUuidGenerator::new(uid1.clone());
//    let mut uid_generator2 = SameUuidGenerator::new(uid2.clone());
//    let mut device_id_generator = SameUuidGenerator::new(device_uuid.clone());
//    client_cmd::register_device_by_uuid_generator(&mut uid_generator1, &mut device_id_generator, &connection).unwrap();
//
//    let second_registration_result =
//        client_cmd::register_device_by_uuid_generator(&mut uid_generator2, &mut device_id_generator, &connection);
//    assert!(second_registration_result.is_err()); // test doesn't make sense otherwise
//
//    let app_user = app_user::select_by_uid(&uid2, &connection);
//    assert!(app_user.unwrap().is_none());
}

#[test]
fn uid_duplication_leaves_google_user_not_created() {
  // TODO
}