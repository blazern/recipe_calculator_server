use std::collections::HashMap;
use std::convert::From;
use std::sync::Mutex;
use serde_json::Value as JsonValue;
use uuid;

use config::Config;
use db::core::migrator;
use db::pool::connection_pool::ConnectionPool;
use db::pool::connection_pool::ConnectionType;
use db::pool::error::Error as PoolError;
use server::error::Error as ServerError;
use server::error::ErrorKind as ServerErrorKind;

use super::error::Error;
use super::constants;
use super::client_cmd;
use super::requests_handler::RequestsHandler;

pub struct RequestsHandlerImpl {
    connection_pool: Mutex<ConnectionPool>,
}

impl RequestsHandlerImpl {
    pub fn new(config: Config) -> Result<RequestsHandlerImpl, Error> {
        let pool = ConnectionPool::new(ConnectionType::UserConnection, config.clone());
        let mut server_user_pool = ConnectionPool::new(ConnectionType::ServerConnection, config);

        let server_user_connection = server_user_pool.borrow()?;
        migrator::perform_migrations(&server_user_connection)?;

        Ok(RequestsHandlerImpl {
            connection_pool: Mutex::new(pool)
        })
    }

    fn handle_impl(&mut self, request: &str, query: Option<&str>) -> Result<JsonValue, RequestError> {
        let args = query_to_args(query)?;

        let mut pool = self.connection_pool.lock().expect("Broken mutex means broken app");
        let connection = (&mut pool).borrow()?;

        match request {
            constants::CMD_REGISTER_USER => {
                let user_name = args.get_or_request_error(constants::ARG_USER_NAME, query)?;
                let social_network_type = args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TYPE, query)?;
                let social_network_token = args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TOKEN, query)?;
                let result = client_cmd::register_user(user_name, social_network_type, social_network_token, &connection)?;
                return Ok(json!({
                    constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
                    constants::FIELD_NAME_USER_ID: result.uid.to_string(),
//                    constants::FIELD_NAME_CLIENT_TOKEN: result.client_token().to_string(),
                }));
            },
            &_ => {
                return Err(RequestError::new(
                    constants::FIELD_STATUS_UNKNOWN_REQUEST,
                    &format!("Unknown request: {}", request)));
            }
        }
    }
}

impl RequestsHandler for RequestsHandlerImpl {
    fn handle(&mut self, request: &str, query: Option<&str>) -> String {
        let result = self.handle_impl(request, query);
        let response = match result {
            Ok(result) => result,
            Err(error) => {
                json!({
                    constants::FIELD_NAME_STATUS: error.status,
                    constants::FIELD_NAME_ERROR_DESCRIPTION: error.error_description
                })
            }
        };
        return response.to_string();
    }
}

fn query_to_args(query: Option<&str>) -> Result<HashMap<&str, &str>, RequestError> {
    let mut result = HashMap::new();

    match query {
        Some(query) => {
            let pairs = query.split("&");
            for pair in pairs {
                let mut key_and_value = pair.split("=");
                let key = key_and_value.next();
                let value = key_and_value.next();
                if key_and_value.next().is_some() {
                    return Err(RequestError::new(
                        constants::FIELD_STATUS_INVALID_QUERY,
                        &format!("invalid key-value pair: {}", pair)));
                }
                match (key, value) {
                    (Some(key), Some(value)) => {
                        result.insert(key, value);
                    }
                    _ => {
                        return Err(RequestError::new(
                            constants::FIELD_STATUS_INVALID_QUERY,
                            &format!("invalid key-value pair: {}", pair)));
                    }
                }
            }
        },
        None => {}
    }

    return Ok(result);
}

struct RequestError {
    status: String,
    error_description: String
}

impl RequestError {
    fn new(status: &str, error_description: &str) -> Self {
        RequestError {
            status: status.to_string(),
            error_description: error_description.to_string()
        }
    }
}

trait HashMapAdditionalOperations {
    fn get_or_request_error<'a>(&'a self, key: &str, request_query: Option<&str>) -> Result<&'a str, RequestError>;
}
impl HashMapAdditionalOperations for HashMap<&str, &str> {
    fn get_or_request_error<'a>(&'a self, key: &str, request_query: Option<&str>) -> Result<&'a str, RequestError> {
        let result = self.get(key);
        return match result {
            Some(result) => Ok(result),
            None => {
                let request_query = match request_query {
                    Some(request_query) => request_query,
                    None => ""
                };
                Err(RequestError::new(
                    constants::FIELD_STATUS_PARAM_MISSING,
                    &format!("No param '{}' in query: {}", key, request_query)))
            }
        };
    }
}

impl From<PoolError> for RequestError {
    fn from(error: PoolError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Pool error: {}", error))
    }
}

impl From<ServerError> for RequestError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError(ServerErrorKind::DeviceNotFoundError(device_id), _) => {
                RequestError::new(
                    constants::FIELD_STATUS_UNKNOWN_DEVICE,
                    &format!("Unknown device, UUID: {}", device_id))
            },
            ServerError(error @ _, _) => {
                RequestError::new(
                    constants::FIELD_STATUS_INTERNAL_ERROR,
                    &format!("Internal DB error: {}", error))
            }
        }
    }
}

impl From<uuid::ParseError> for RequestError {
    fn from(error: uuid::ParseError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INVALID_UUID,
            &format!("Invalid UUID: {}", error))
    }
}

#[cfg(test)]
#[path = "./requests_handler_impl_test.rs"]
mod requests_handler_impl_test;