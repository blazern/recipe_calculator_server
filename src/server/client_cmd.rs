use diesel::pg::PgConnection;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error::DatabaseError;
use uuid::Uuid;

use db::app_user;
use db::device;
use error::Error;
use error::ErrorKind::DieselError;
use error::ErrorKind::DeviceIdAlreadyExists;

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
// Returns error::ErrorKind::DeviceIdAlreadyExists when receives already used Device ID.
pub fn register_device(device_id: Uuid, db_connection: &PgConnection) -> Result<(), Error> {
    return register_device_by_uid_generator(&mut DefaultUserAppUidGenerator{}, device_id, &db_connection);
}

fn register_device_by_uid_generator(
        generator: &mut UserAppUidGenerator, device_id: Uuid, db_connection: &PgConnection) -> Result<(), Error> {
    let app_user = create_app_user_by_uid_generator(generator, &db_connection)?;
    let device = device::insert(device::new(device_id, &app_user), &db_connection);
    match device {
        Ok(_) => {
            return Ok(());
        },
        Err(Error(DieselError(DatabaseError(DatabaseErrorKind::UniqueViolation, _)), _)) => {
            return Err(DeviceIdAlreadyExists(device_id).into());
        }
        Err(error) => {
            return Err(error);
        }
    };
}

fn create_app_user_by_uid_generator(
        uuid_generator: &mut UserAppUidGenerator, db_connection: &PgConnection) -> Result<app_user::AppUser, Error> {

    let mut app_user_result: Result<app_user::AppUser, Error> = Err("No user insertion result".into());
    for _ in 0..DUPLICATED_APP_USER_UID_MAX_STREAK {
        // Note: while very unlikely,
        // the 'new_v4' function can lead to uids collisions.
        // That's why uid generation is performed in the loop.
        let uid = uuid_generator.generate();
        app_user_result = app_user::insert(app_user::new(uid), &db_connection);
        match app_user_result {
            Ok(app_user) => {
                return Ok(app_user);
            },
            Err(Error(DieselError(DatabaseError(DatabaseErrorKind::UniqueViolation, _)), _)) => {
                continue;
            }
            Err(error) => {
                return Err(error);
            }
        };
    }
    // If we're here, DUPLICATED_APP_USER_UID_MAX_STREAK was exceeded.
    return app_user_result;
}

#[cfg(test)]
#[path = "./client_cmd_test.rs"]
mod client_cmd_test;