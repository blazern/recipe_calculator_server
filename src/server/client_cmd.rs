use uuid::Uuid;

use super::error::Error;
use super::error::ErrorKind::DeviceIdDuplicationError;
use super::error::ErrorKind::AppUserUniqueIdCreationError;
use db::app_user;
use db::connection::DBConnection;
use db::device;
use db::error::Error as DBError;
use db::error::ErrorKind as DBErrorKind;

const DUPLICATED_APP_USER_UID_MAX_STREAK: i32 = 5;

pub trait UserAppUidGenerator {
    fn generate(&mut self) -> Uuid;
}

struct DefaultUserAppUidGenerator;

impl UserAppUidGenerator for DefaultUserAppUidGenerator {
    fn generate(&mut self) -> Uuid {
        return Uuid::new_v4();
    }
}

// Registers new device by creating an 'app_user::AppUser' for it and storing
// both device and app_user to DB.
//
// Returns DeviceIdDuplication when receives already used Device ID.
pub fn register_device(device_id: Uuid, db_connection: &DBConnection) -> Result<(), Error> {
    return register_device_by_uid_generator(&mut DefaultUserAppUidGenerator{}, device_id, &db_connection);
}

fn register_device_by_uid_generator(
        generator: &mut UserAppUidGenerator, device_id: Uuid, db_connection: &DBConnection) -> Result<(), Error> {
    let app_user = create_app_user_by_uid_generator(generator, &db_connection)?;
    let device = device::insert(device::new(device_id, &app_user), &db_connection);

    match device {
        Ok(_) => {
            return Ok(());
        },
        Err(error @ DBError(DBErrorKind::UniqueViolation(_), _)) => {
            return Err(DeviceIdDuplicationError(device_id, error).into());
        }
        Err(error) => {
            return Err(error.into());
        }
    };
}

fn create_app_user_by_uid_generator(
        uuid_generator: &mut UserAppUidGenerator,
        db_connection: &DBConnection) -> Result<app_user::AppUser, Error> {

    let mut app_user_result: Result<app_user::AppUser, DBError>;

    for index in 1..DUPLICATED_APP_USER_UID_MAX_STREAK+1 {
        // Note: while very unlikely,
        // the 'new_v4' function can lead to uids collisions.
        // That's why uid generation is performed in a loop.
        let uid = uuid_generator.generate();
        app_user_result = app_user::insert(app_user::new(uid), &db_connection);
        match (app_user_result, index) {
            (Ok(app_user), _) => {
                return Ok(app_user);
            },
            (Err(error @ DBError(DBErrorKind::UniqueViolation(_), _)), DUPLICATED_APP_USER_UID_MAX_STREAK) => {
                // If index reached max value - return unique ID creation error
                return Err(AppUserUniqueIdCreationError(error).into());
            }
            (Err(DBError(DBErrorKind::UniqueViolation(_), _)), _) => {
                // If index isn't DUPLICATED_APP_USER_UID_MAX_STREAK, yet - continue trying.
                continue;
            }
            (Err(error), _) => {
                // On any error other than UniqueViolation - fail immediately.
                return Err(error.into());
            }
        };
    }
    panic!("Expected to be not reached");
}

#[cfg(test)]
#[path = "./client_cmd_test.rs"]
mod client_cmd_test;