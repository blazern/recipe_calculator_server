extern crate diesel;
extern crate uuid;

use std::str::FromStr;

use uuid::Uuid;

use server::error::Error;
use server::error::ErrorKind::DeviceIdDuplicationError;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::device;
use server::client_cmd;

include!("../testing_config.rs.inc");

struct SameUidGenerator {
    uid: Uuid,
    generations_count: i32,
}

impl SameUidGenerator {
    fn new(uid: Uuid) -> SameUidGenerator {
        SameUidGenerator{ uid, generations_count: 0 }
    }
}

impl client_cmd::UserAppUidGenerator for SameUidGenerator {
    fn generate(&mut self) -> Uuid {
        self.generations_count += 1;
        return self.uid.clone();
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
    let uid = Uuid::from_str("50008400-e29b-41d4-a716-446655440000").unwrap();
    let device_uuid = Uuid::from_str("60008400-e29b-41d4-a716-446655440000").unwrap();
    delete_device_with(&device_uuid);
    delete_app_user_with(&uid);

    let config = get_testing_config();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator = SameUidGenerator::new(uid.clone());
    client_cmd::register_device_by_uid_generator(&mut uid_generator, device_uuid.clone(), &connection).unwrap();

    // making sure device is inserted
    device::select_by_uuid(&device_uuid, &connection).unwrap().unwrap();
    // making sure app_user is inserted
    app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
}

#[test]
fn device_registration_returns_device_id_duplication_error_on_duplication() {
    let uid1 = Uuid::from_str("50008400-e29b-41d4-a716-446655440001").unwrap();
    let uid2 = Uuid::from_str("50008400-e29b-41d4-a716-446655440002").unwrap();
    let device_uuid = Uuid::from_str("60008400-e29b-41d4-a716-446655440001").unwrap();
    delete_device_with(&device_uuid);
    delete_app_user_with(&uid1);
    delete_app_user_with(&uid2);

    let config = get_testing_config();
    let connection = DBConnection::for_client_user(&config).unwrap();

    let mut uid_generator1 = SameUidGenerator::new(uid1.clone());
    let mut uid_generator2 = SameUidGenerator::new(uid2.clone());
    client_cmd::register_device_by_uid_generator(&mut uid_generator1, device_uuid.clone(), &connection).unwrap();

    let second_registration_result =
        client_cmd::register_device_by_uid_generator(&mut uid_generator2, device_uuid.clone(), &connection);
    match second_registration_result {
        Err(Error(DeviceIdDuplicationError(_, _), _)) => {
             // OK
        }
        Err(err) => {
            panic!("Expected error::DeviceIdAlreadyExists, but got another error: {:?}", err);
        }
        Ok(_) => {
            panic!("Expected error::DeviceIdAlreadyExists, but registration was successful!");
        }
    }
}

#[test]
fn app_user_creation_is_repeated_on_uid_collisions() {
    let uid = Uuid::from_str("50008400-e29b-41d4-a716-446655440003").unwrap();
    delete_app_user_with(&uid);

    let config = get_testing_config();
    let connection = DBConnection::for_client_user(&config).unwrap();

    // Ensuring that AppUser with used uid already exists in DB
    let inserted_app_user = app_user::insert(app_user::new(uid.clone()), &connection);
    // Stop compiler's unused var warning
    assert!(inserted_app_user.is_ok() || inserted_app_user.is_err());

    let mut uid_generator = SameUidGenerator::new(uid);
    {
        let app_creation_result = client_cmd::create_app_user_by_uid_generator(&mut uid_generator, &connection);
        assert!(app_creation_result.is_err());
    }

    assert_eq!(client_cmd::DUPLICATED_APP_USER_UID_MAX_STREAK, uid_generator.generations_count)
}
