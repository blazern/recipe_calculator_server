use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;
use std::sync::Mutex;
use serde_json::Value as JsonValue;
use uuid;
use uuid::Uuid;

use config::Config;
use db::pool::connection_pool::ConnectionPool;
use db::pool::connection_pool::ConnectionType;
use db::pool::error::Error as PoolError;
use server::error::Error as ServerError;
use server::error::ErrorKind as ServerErrorKind;

use super::constants;
use super::client_cmd;
use super::requests_handler::RequestsHandler;

struct Error {
    status: String,
    error_description: String
}

impl Error {
    fn new(status: &str, error_description: &str) -> Self {
        Error {
            status: status.to_string(),
            error_description: error_description.to_string()
        }
    }
}

pub struct RequestsHandlerImpl {
    connection_pool: Mutex<ConnectionPool>,
}



impl RequestsHandlerImpl {
    pub fn new(config: Config) -> RequestsHandlerImpl {
        let pool = ConnectionPool::new(ConnectionType::UserConnection, config);
        RequestsHandlerImpl{ connection_pool: Mutex::new(pool) }
    }

    fn handle_impl(&mut self, request: &str, query: Option<&str>) -> Result<JsonValue, Error> {
        let args = query_to_args(query)?;

        let mut pool = self.connection_pool.lock().expect("Broken mutex means broken app");
        let connection = (&mut pool).borrow()?;

        match request {
            constants::CMD_REGISTER_DEVICE => {
                let result = client_cmd::register_device(&connection)?;
                return Ok(json!({
                    constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
                    constants::FIELD_NAME_DEVICE_ID: result.to_string()
                }));
            },
            constants::CMD_IS_DEVICE_REGISTERED => {
                let device_uuid = args.get(constants::ARG_DEVICE_ID);
                let device_uuid = match device_uuid {
                    Some(device_uuid) => device_uuid,
                    None => {
                        let query = match query {
                            Some(query) => query,
                            None => ""
                        };
                        return Err(Error::new(
                            constants::FIELD_STATUS_INVALID_QUERY,
                            &format!("No device ID in query: {}", query)));
                    }
                };
                let device_uuid = Uuid::from_str(&device_uuid)?;
                let is_registered = client_cmd::is_device_registered(&connection, &device_uuid)?;

                return Ok(json!({
                    constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
                    constants::FIELD_NAME_REGISTERED: is_registered
                }));
            },
            &_ => {
                return Err(Error::new(
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

fn query_to_args(query: Option<&str>) -> Result<HashMap<&str, &str>, Error> {
    let mut result = HashMap::new();

    match query {
        Some(query) => {
            let pairs = query.split("&");
            for pair in pairs {
                let mut key_and_value = pair.split("=");
                let key = key_and_value.next();
                let value = key_and_value.next();
                if key_and_value.next().is_some() {
                    return Err(Error::new(
                        constants::FIELD_STATUS_INVALID_QUERY,
                        &format!("invalid key-value pair: {}", pair)));
                }
                match (key, value) {
                    (Some(key), Some(value)) => {
                        result.insert(key, value);
                    }
                    _ => {
                        return Err(Error::new(
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

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Error::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Pool error: {}", error))
    }
}

impl From<ServerError> for Error {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError(ServerErrorKind::DeviceNotFoundError(device_id), _) => {
                Error::new(
                    constants::FIELD_STATUS_UNKNOWN_DEVICE,
                    &format!("Unknown device, UUID: {}", device_id))
            },
            ServerError(error @ _, _) => {
                Error::new(
                    constants::FIELD_STATUS_INTERNAL_ERROR,
                    &format!("Internal DB error: {}", error))
            }
        }
    }
}

impl From<uuid::ParseError> for Error {
    fn from(error: uuid::ParseError) -> Self {
        Error::new(
            constants::FIELD_STATUS_INVALID_UUID,
            &format!("Invalid UUID: {}", error))
    }
}

#[cfg(test)]
#[path = "./requests_handler_impl_test.rs"]
mod requests_handler_impl_test;