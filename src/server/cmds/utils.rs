use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::app_user::AppUser;
use crate::db::core::connection::DBConnection;
use crate::db::core::fcm_token;
use crate::db::core::transaction;
use crate::db::pool::connection_pool::ConnectionPool;

use crate::config::Config;
use crate::outside::fcm;
use crate::outside::http_client::HttpClient;
use crate::server::constants;
use crate::server::request_error::RequestError;

pub trait HashMapAdditionalOperations {
    fn get_or_request_error(&self, key: &str) -> Result<String, RequestError>;
    fn get_or_empty(&self, key: &str) -> String;
}

#[allow(clippy::implicit_hasher)]
impl HashMapAdditionalOperations for HashMap<std::string::String, std::string::String> {
    fn get_or_request_error(&self, key: &str) -> Result<String, RequestError> {
        let result = self.get(key);
        match result {
            Some(result) => Ok(result.to_string()),
            None => Err(RequestError::new(
                constants::FIELD_STATUS_PARAM_MISSING.to_owned(),
                format!("No param '{}' in query", key),
            )),
        }
    }
    fn get_or_empty(&self, key: &str) -> String {
        let result = self.get(key);
        match result {
            Some(result) => result.to_string(),
            None => "".to_string(),
        }
    }
}

#[allow(clippy::implicit_hasher)]
pub fn extract_user_from_query_args(
    args: &HashMap<String, String>,
    connection: &dyn DBConnection,
) -> Result<app_user::AppUser, RequestError> {
    let user_id = args.get_or_request_error(constants::ARG_USER_ID)?;
    let user_id = Uuid::parse_str(&user_id)?;
    let client_token = args.get_or_request_error(constants::ARG_CLIENT_TOKEN)?;
    let client_token = Uuid::parse_str(&client_token)?;

    let user = app_user::select_by_uid(&user_id, connection)?;
    match user {
        Some(user) => {
            if *user.client_token() != client_token {
                Err(RequestError::new(
                    constants::FIELD_STATUS_INVALID_CLIENT_TOKEN.to_owned(),
                    "Given client token doesn't belong to given user".to_owned(),
                ))
            } else {
                Ok(user)
            }
        }
        None => Err(RequestError::new(
            constants::FIELD_STATUS_USER_NOT_FOUND.to_owned(),
            "User with given user ID not found".to_owned(),
        )),
    }
}

pub fn db_transaction<T, F>(connection: &dyn DBConnection, action: F) -> Result<T, RequestError>
where
    F: FnOnce() -> Result<T, RequestError>,
{
    transaction::start::<T, RequestError, _>(connection, action)
}

/// Returns future which resolves when a notification is sent to the |user|.
/// Or immediately if the user doesn't have a FCM-token.
pub async fn notify_user(
    user: &AppUser,
    msg: String,
    connections_pool: ConnectionPool,
    config: &Config,
    fcm_address: &str,
    http_client: Arc<HttpClient>,
) -> Result<(), RequestError> {
    let mut connections_pool = connections_pool;
    let connection = connections_pool.borrow_connection()?;

    let fcm_token = fcm_token::select_by_user_id(user.id(), &connection)?;
    let fcm_token = if let Some(fcm_token) = fcm_token {
        fcm_token
    } else {
        return Ok(());
    };

    // TODO: log all possible errors from send
    let send_res = fcm::send(
        msg,
        fcm_token.token_value(),
        config.fcm_server_token(),
        fcm_address,
        http_client,
    )
    .await?;

    if let fcm::SendResult::Error(error) = send_res {
        Err(RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR.to_owned(),
            format!("FCM error: {}", error),
        ))
    } else {
        Ok(())
    }
}
