use futures;
use futures::Future;
use uuid::Uuid;
use std::sync::Arc;

use super::error::Error;
use super::error::ErrorKind::UniqueUuidCreationError;
use config::Config;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::error::Error as DBError;
use db::core::error::ErrorKind as DBErrorKind;
use db::core::transaction;
use http_client::HttpClient;

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

pub fn register_user<Conn>(user_name: String,
//                     social_network_type: String,
//                     social_network_token: String,
                     config: Config,
                     db_connection: Conn,
                     http_client: Arc<HttpClient>)
        -> impl Future<Item=UserRegistrationResult, Error=Error>
        where Conn: DBConnection {
    return register_user_by_uuid_generator(
        user_name,
//        social_network_type,
//        social_network_token,
        DefaultUuidGenerator{},
        config,
        db_connection,
        http_client);
}

fn register_user_by_uuid_generator<Conn>(
        user_name: String,
//        social_network_type: String,
//        social_network_token: String,
        uid_generator: impl UuidGenerator,
        _config: Config,
        db_connection: Conn,
        _http_client: Arc<HttpClient>)
            -> impl Future<Item=UserRegistrationResult, Error=Error>
                where Conn: DBConnection {
//    return futures::future::ok::<String, Error>(social_network_type)
//        .and_then(|social_network_type| {
//            let result: Result<(), Error> = match social_network_type.as_ref() {
//                "vk" => { Ok(()) },
//                _ => { Err(UnsupportedSocialNetwork(social_network_type).into()) }
//            };
//            futures::done(result)
//        })
//        .and_then(move |_| {
//            vk::check_token(config.vk_server_token(), &social_network_token, http_client)
//                .map_err(|err| err.into())
//        })
//        .and_then(move |vk_check_result| {
//            if !vk_check_result.is_success() {
//                let result = Err(VKTokenCheckError(
//                    vk_check_result.error_msg().as_ref().expect("check result is success").clone(),
//                    vk_check_result.error_code().as_ref().expect("check result is success").clone()).into());
//                return futures::done(result);
//            }
    return futures::lazy(move || {
            let db_connection_ref = &db_connection;
            let mut uid_generator = uid_generator;

            let result = transaction::start(&db_connection, move || {
                let uid = uid_generator.generate();
                let app_user = app_user::insert(app_user::new(uid, &user_name), db_connection_ref);
                let app_user = app_user.map_err(extract_uuid_duplication_error)?;

//                let vk_user =
//                    vk_user::new(
//                        vk_check_result.user_id().as_ref().expect("check result is success").clone(),
//                        &app_user);
//                vk_user::insert(vk_user,
//                                &db_connection_ref)?;

                return Ok(UserRegistrationResult {
                    uid: app_user.uid().clone(),
                });
            });
            futures::done(result)
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