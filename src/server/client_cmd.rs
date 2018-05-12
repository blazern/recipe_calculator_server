use uuid::Uuid;

use super::error::Error;
use super::error::ErrorKind::DeviceNotFoundError;
use super::error::ErrorKind::UniqueUuidCreationError;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::device;
use db::core::error::Error as DBError;
use db::core::error::ErrorKind as DBErrorKind;
use db::core::transaction;

pub trait UuidGenerator {
    fn generate(&mut self) -> Uuid;
}

struct DefaultUuidGenerator;

impl UuidGenerator for DefaultUuidGenerator {
    fn generate(&mut self) -> Uuid {
        return Uuid::new_v4();
    }
}

// Registers new device by creating an 'app_user::AppUser' for it and storing
// both device and app_user to DB.
//
pub fn register_device(db_connection: &DBConnection) -> Result<Uuid, Error> {
    return register_device_by_uuid_generator(
        &mut DefaultUuidGenerator{}, &mut DefaultUuidGenerator{}, &db_connection);
}

fn register_device_by_uuid_generator(
        uid_generator: &mut UuidGenerator,
        device_id_generator: &mut UuidGenerator,
        db_connection: &DBConnection) -> Result<Uuid, Error> {
    let device_id = device_id_generator.generate();
    let uid = uid_generator.generate();

    return transaction::start(&db_connection, || {
        let app_user = app_user::insert(app_user::new(uid), &db_connection);
        let app_user = app_user.map_err(extract_uuid_duplication_error)?;
        let device = device::insert(device::new(device_id, &app_user), &db_connection);
        let device = device.map_err(extract_uuid_duplication_error)?;
        return Ok(device.uuid().clone());
    });
}

fn extract_uuid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        error @ DBError(DBErrorKind::UniqueViolation(_), _) => {
            return UniqueUuidCreationError(error).into();
        }
        error => {
            return error.into();
        }
    }
}

pub fn is_device_registered(db_connection: &DBConnection, device_id: &Uuid) -> Result<bool, Error> {
    let _device = device::select_by_uuid(&device_id, &db_connection);
    match _device {
        Ok(Some(_device)) => {
            return Ok(true);
        },
        Ok(None) => {
            return Ok(false);
        },
        Err(_) => {
            return Err(DeviceNotFoundError(device_id.clone()).into());
        }
    }
}

#[cfg(test)]
#[path = "./client_cmd_test.rs"]
mod client_cmd_test;