use uuid::Uuid;

use super::error::Error;
use super::error::ErrorKind::UniqueUuidCreationError;
use db::core::app_user;
use db::core::connection::DBConnection;
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

pub struct UserRegistrationResult {
    pub uid: Uuid,
}

pub fn register_user(user_name: &str,
                     social_network_type: &str,
                     social_network_token: &str,
                     db_connection: &DBConnection) -> Result<UserRegistrationResult, Error> {
    return register_user_by_uuid_generator(
        user_name,
        social_network_type,
        social_network_token,
        &mut DefaultUuidGenerator{},
        &db_connection);
}

fn register_user_by_uuid_generator(
        user_name: &str,
        _social_network_type: &str,
        _social_network_token: &str,
        uid_generator: &mut UuidGenerator,
        db_connection: &DBConnection) -> Result<UserRegistrationResult, Error> {
    // TODO: check social network type, create an according entry in DB
    let uid = uid_generator.generate();

    return transaction::start(&db_connection, || {
        let app_user = app_user::insert(app_user::new(uid, user_name), &db_connection);
        let app_user = app_user.map_err(extract_uuid_duplication_error)?;
        return Ok(UserRegistrationResult {
            uid: app_user.uid().clone(),
        });
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

#[cfg(test)]
#[path = "./client_cmd_test.rs"]
mod client_cmd_test;