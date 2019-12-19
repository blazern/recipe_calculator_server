use futures;
use futures::done;
use futures::future::ok;
use futures::Future;
use std::sync::Arc;
use uuid::Uuid;

use super::error::Error;
use super::error::ErrorKind::UniqueUuidCreationError;
use super::error::ErrorKind::UnsupportedSocialNetwork;
use super::error::ErrorKind::VKTokenCheckError;
use super::error::ErrorKind::VKTokenCheckFail;
use super::error::ErrorKind::VkUidDuplicationError;
use super::user_data_generators::new_user_uuid_generator_for;
use super::user_data_generators::new_vk_token_checker_for;
use super::user_data_generators::UserUuidGenerator;
use super::user_data_generators::VkTokenChecker;
use config::Config;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::error::Error as DBError;
use db::core::error::ErrorKind as DBErrorKind;
use db::core::transaction;
use db::core::vk_user;
use http_client::HttpClient;
use outside::vk;

pub struct UserRegistrationResult {
    pub uid: Uuid,
    pub client_token: Uuid,
}

pub fn register_user<Conn>(
    user_name: String,
    social_network_type: String,
    social_network_token: String,
    overrides: String,
    config: Config,
    db_connection: Conn,
    http_client: Arc<HttpClient>,
) -> impl Future<Item = UserRegistrationResult, Error = Error> + Send
where
    Conn: DBConnection + Send,
{
    let user_uuid_generator = new_user_uuid_generator_for(&overrides);
    let vk_token_checker = new_vk_token_checker_for(
        &overrides,
        social_network_token,
        config.vk_server_token().to_string(),
        http_client,
    );

    register_user_impl(
        user_name,
        social_network_type,
        db_connection,
        user_uuid_generator,
        vk_token_checker,
    )
}

fn register_user_impl<Conn>(
    user_name: String,
    social_network_type: String,
    db_connection: Conn,
    user_uuid_generator: Box<dyn UserUuidGenerator>,
    vk_token_checker: Box<dyn VkTokenChecker>,
) -> impl Future<Item = UserRegistrationResult, Error = Error> + Send
where
    Conn: DBConnection + Send,
{
    ok((social_network_type, vk_token_checker))
        .and_then(|(social_network_type, vk_token_checker)| {
            let result = match social_network_type.as_ref() {
                "vk" => Ok(vk_token_checker),
                _ => Err(UnsupportedSocialNetwork(social_network_type).into()),
            };
            done(result)
        })
        .and_then(|vk_token_checker| vk_token_checker.check_token().map_err(|err| err.into()))
        .and_then(|vk_check_result| {
            let result = match vk_check_result {
                vk::CheckResult::Success { user_id } => Ok(user_id),
                vk::CheckResult::Fail => Err(VKTokenCheckFail.into()),
                vk::CheckResult::Error {
                    error_code,
                    error_msg,
                } => Err(VKTokenCheckError(error_code, error_msg).into()),
            };
            futures::done(result)
        })
        .and_then(move |vk_uid| {
            let db_connection_ref = &db_connection;

            let result = transaction::start(&db_connection, move || {
                let uid = user_uuid_generator.generate();
                let client_token = Uuid::new_v4();

                let app_user = app_user::insert(
                    app_user::new(uid, user_name.to_string(), client_token),
                    db_connection_ref,
                );
                let app_user = app_user.map_err(extract_uuid_duplication_error)?;

                let vk_user = vk_user::new(vk_uid, &app_user);
                let vk_user_insertion = vk_user::insert(vk_user, db_connection_ref);
                vk_user_insertion.map_err(extract_vk_uid_duplication_error)?;

                Ok(UserRegistrationResult {
                    uid: *app_user.uid(),
                    client_token: *app_user.client_token(),
                })
            });
            done(result)
        })
}

fn extract_uuid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        error @ DBError(DBErrorKind::UniqueViolation(_), _) => {
            UniqueUuidCreationError(error).into()
        }
        error => error.into(),
    }
}

fn extract_vk_uid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        DBError(DBErrorKind::UniqueViolation(_), _) => VkUidDuplicationError {}.into(),
        error => error.into(),
    }
}
