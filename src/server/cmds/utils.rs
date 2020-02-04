use std::collections::HashMap;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::connection::DBConnection;
use crate::db::core::transaction;

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
                constants::FIELD_STATUS_PARAM_MISSING,
                &format!("No param '{}' in query", key),
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
                    constants::FIELD_STATUS_INVALID_CLIENT_TOKEN,
                    "Given client token doesn't belong to given user",
                ))
            } else {
                Ok(user)
            }
        }
        None => Err(RequestError::new(
            constants::FIELD_STATUS_USER_NOT_FOUND,
            "User with given user ID not found",
        )),
    }
}

pub fn db_transaction<T, F>(connection: &dyn DBConnection, action: F) -> Result<T, RequestError>
where
    F: FnOnce() -> Result<T, RequestError>,
{
    transaction::start::<T, RequestError, _>(connection, action)
}
