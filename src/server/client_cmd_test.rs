extern crate diesel;
extern crate uuid;

use std::str::FromStr;

use uuid::Uuid;

use server::error::Error;
use server::error::ErrorKind::UniqueUuidCreationError;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::device;
use server::client_cmd;
use testing_config;

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
    let connection = DBConnection::for_admin_user().unwrap();

    let selected_app_user = app_user::select_by_uid(&uid, &connection);
    match selected_app_user {
        Ok(Some(app_user)) => {
            app_user::delete_by_id(app_user.id(), &connection).unwrap();
        }
        _ => {}
    }
}

fn delete_device_with(device_uuid: &Uuid) {
    let connection = DBConnection::for_admin_user().unwrap();

    let selected_device = device::select_by_uuid(&device_uuid, &connection);
    match selected_device {
        Ok(Some(device)) => {
            device::delete_by_id(device.id(), &connection).unwrap();
        }
        _ => {}
    }
}

#[test]
fn device_registration_works() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000000").unwrap();
    let device_uuid = Uuid::from_str("00000000-0000-0000-0000-001000000001").unwrap();
    delete_device_with(&device_uuid);
    delete_app_user_with(&uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator = SameUuidGenerator::new(uid.clone());
    let mut device_id_generator = SameUuidGenerator::new(device_uuid.clone());
    client_cmd::register_device_by_uuid_generator(&mut uid_generator, &mut device_id_generator, &connection).unwrap();

    // making sure device is inserted
    device::select_by_uuid(&device_uuid, &connection).unwrap().unwrap();
    // making sure app_user is inserted
    app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
}

#[test]
fn device_registration_returns_duplication_error_on_device_id_duplication() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-001000000002").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-001000000003").unwrap();
    let device_uuid = Uuid::from_str("00000000-0000-0000-0000-001000000004").unwrap();
    delete_device_with(&device_uuid);
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator1 = SameUuidGenerator::new(uid1.clone());
    let mut uid_generator2 = SameUuidGenerator::new(uid2.clone());
    let mut device_id_generator = SameUuidGenerator::new(device_uuid.clone());
    client_cmd::register_device_by_uuid_generator(&mut uid_generator1, &mut device_id_generator, &connection).unwrap();

    let second_registration_result =
        client_cmd::register_device_by_uuid_generator(&mut uid_generator2, &mut device_id_generator, &connection);
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
fn device_registration_returns_duplication_error_on_uid_duplication() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000005").unwrap();
    let device_uuid1 = Uuid::from_str("00000000-0000-0000-0000-001000000006").unwrap();
    let device_uuid2 = Uuid::from_str("00000000-0000-0000-0000-001000000007").unwrap();
    delete_device_with(&device_uuid1);
    delete_device_with(&device_uuid2);
    delete_app_user_with(&uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator = SameUuidGenerator::new(uid.clone());
    let mut device_id_generator1 = SameUuidGenerator::new(device_uuid1.clone());
    let mut device_id_generator2 = SameUuidGenerator::new(device_uuid2.clone());
    client_cmd::register_device_by_uuid_generator(&mut uid_generator, &mut device_id_generator1, &connection).unwrap();

    let second_registration_result =
        client_cmd::register_device_by_uuid_generator(&mut uid_generator, &mut device_id_generator2, &connection);
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
fn device_id_duplication_leaves_user_not_created() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-001000000008").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-001000000009").unwrap();
    let device_uuid = Uuid::from_str("00000000-0000-0000-0000-001000000010").unwrap();
    delete_device_with(&device_uuid);
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator1 = SameUuidGenerator::new(uid1.clone());
    let mut uid_generator2 = SameUuidGenerator::new(uid2.clone());
    let mut device_id_generator = SameUuidGenerator::new(device_uuid.clone());
    client_cmd::register_device_by_uuid_generator(&mut uid_generator1, &mut device_id_generator, &connection).unwrap();

    let second_registration_result =
        client_cmd::register_device_by_uuid_generator(&mut uid_generator2, &mut device_id_generator, &connection);
    assert!(second_registration_result.is_err()); // test doesn't make sense otherwise

    let app_user = app_user::select_by_uid(&uid2, &connection);
    assert!(app_user.unwrap().is_none());
}

#[test]
fn uid_duplication_leaves_device_not_created() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000011").unwrap();
    let device_uuid1 = Uuid::from_str("00000000-0000-0000-0000-001000000012").unwrap();
    let device_uuid2 = Uuid::from_str("00000000-0000-0000-0000-001000000013").unwrap();
    delete_device_with(&device_uuid1);
    delete_device_with(&device_uuid2);
    delete_app_user_with(&uid);

    let config = testing_config::get();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator = SameUuidGenerator::new(uid.clone());
    let mut device_id_generator1 = SameUuidGenerator::new(device_uuid1.clone());
    let mut device_id_generator2 = SameUuidGenerator::new(device_uuid2.clone());
    client_cmd::register_device_by_uuid_generator(&mut uid_generator, &mut device_id_generator1, &connection).unwrap();

    let second_registration_result =
        client_cmd::register_device_by_uuid_generator(&mut uid_generator, &mut device_id_generator2, &connection);
    assert!(second_registration_result.is_err()); // test doesn't make sense otherwise

    let device = device::select_by_uuid(&device_uuid2, &connection);
    assert!(device.unwrap().is_none());
}