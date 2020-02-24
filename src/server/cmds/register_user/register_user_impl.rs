use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::db::core::app_user;
use crate::db::core::connection::DBConnection;
use crate::db::core::error::Error as DBError;
use crate::db::core::error::ErrorKind as DBErrorKind;
use crate::db::core::gp_user;
use crate::db::core::transaction;
use crate::db::core::vk_user;
use crate::outside::http_client::HttpClient;
use crate::server::error::Error;
use crate::server::error::ErrorKind::GPUidDuplicationError;
use crate::server::error::ErrorKind::UniqueUuidCreationError;
use crate::server::error::ErrorKind::VKUidDuplicationError;

use super::social_network_token_check::check_token;
use super::social_network_token_check::TokenCheckSuccess;
use super::user_data_generators::new_user_uuid_generator_for;
use super::user_data_generators::UserUuidGenerator;

pub struct UserRegistrationResult {
    pub uid: Uuid,
    pub client_token: Uuid,
}

pub async fn register_user<Conn>(
    user_name: String,
    social_network_type: String,
    social_network_token: String,
    overrides: String,
    config: Config,
    db_connection: Conn,
    http_client: Arc<HttpClient>,
) -> Result<UserRegistrationResult, Error>
where
    Conn: DBConnection + Send,
{
    let user_uuid_generator = new_user_uuid_generator_for(&overrides);

    register_user_impl(
        user_name,
        db_connection,
        user_uuid_generator,
        social_network_type,
        social_network_token,
        overrides,
        config,
        http_client,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn register_user_impl<Conn>(
    user_name: String,
    db_connection: Conn,
    user_uuid_generator: Box<dyn UserUuidGenerator>,
    social_network_type: String,
    social_network_token: String,
    overrides: String,
    config: Config,
    http_client: Arc<HttpClient>,
) -> Result<UserRegistrationResult, Error>
where
    Conn: DBConnection + Send,
{
    let checked_token = check_token(
        social_network_type,
        social_network_token,
        &overrides,
        http_client,
        config,
    )
    .await?;
    let db_connection_ref = &db_connection;

    transaction::start(&db_connection, move || {
        let uid = user_uuid_generator.generate();
        let client_token = Uuid::new_v4();

        let app_user = app_user::insert(
            app_user::new(uid, user_name.to_string(), client_token),
            db_connection_ref,
        );
        let app_user = app_user.map_err(extract_uuid_duplication_error)?;

        match checked_token {
            TokenCheckSuccess::VK { uid } => {
                let vk_user = vk_user::new(uid, &app_user);
                let vk_user_insertion = vk_user::insert(vk_user, db_connection_ref);
                vk_user_insertion.map_err(extract_vk_uid_duplication_error)?;
            }
            TokenCheckSuccess::GP { uid } => {
                let gp_user = gp_user::new(uid, &app_user);
                let gp_user_insertion = gp_user::insert(gp_user, db_connection_ref);
                gp_user_insertion.map_err(extract_gp_uid_duplication_error)?;
            }
        };

        Ok(UserRegistrationResult {
            uid: *app_user.uid(),
            client_token: *app_user.client_token(),
        })
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
        DBError(DBErrorKind::UniqueViolation(_), _) => VKUidDuplicationError {}.into(),
        error => error.into(),
    }
}

fn extract_gp_uid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        DBError(DBErrorKind::UniqueViolation(_), _) => GPUidDuplicationError {}.into(),
        error => error.into(),
    }
}
