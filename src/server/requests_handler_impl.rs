use futures::done;
use futures::future::err;
use futures::future::ok;
use futures::Future;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::convert::From;
use std::sync::Arc;
use std::sync::Mutex;
use uuid;

use config::Config;
use db::core::migrator;
use db::pool::connection_pool::BorrowedDBConnection;
use db::pool::connection_pool::ConnectionPool;
use db::pool::connection_pool::ConnectionType;
use db::pool::error::Error as PoolError;
use http_client::HttpClient;
use server::error::Error as ServerError;
use server::error::ErrorKind as ServerErrorKind;

use super::client_cmd;
use super::constants;
use super::error::Error;
use super::requests_handler::RequestsHandler;

pub struct RequestsHandlerImpl {
    connection_pool: Mutex<ConnectionPool>,
    config: Config,
    http_client: Arc<HttpClient>,
}

impl RequestsHandlerImpl {
    pub fn new(config: Config) -> Result<RequestsHandlerImpl, Error> {
        let pool = ConnectionPool::new(ConnectionType::UserConnection, config.clone());
        let mut server_user_pool =
            ConnectionPool::new(ConnectionType::ServerConnection, config.clone());

        let server_user_connection = server_user_pool.borrow()?;
        migrator::perform_migrations(&server_user_connection)?;

        Ok(RequestsHandlerImpl {
            connection_pool: Mutex::new(pool),
            config,
            http_client: Arc::new(HttpClient::new()?),
        })
    }

    fn handle_impl(
        &mut self,
        request: String,
        query: String,
    ) -> impl Future<Item = JsonValue, Error = RequestError> + Send {
        let mut pool = self
            .connection_pool
            .lock()
            .expect("Broken mutex means broken app");
        let connection = (&mut pool).borrow();
        let config = self.config.clone();
        let http_client = self.http_client.clone();

        done(connection)
            .map_err(|err| err.into())
            .map(|connection| (request, query, connection))
            .and_then(|(request, query, connection)| {
                done(query_to_args(query)).map(|args| (request, args, connection))
            })
            .and_then(|(request, args, connection)| {
                handle_prepared_request(request, args, connection, config, http_client)
            })
    }
}

fn handle_prepared_request(
    request: String,
    args: HashMap<String, String>,
    connection: BorrowedDBConnection,
    config: Config,
    http_client: Arc<HttpClient>,
) -> Box<dyn Future<Item = JsonValue, Error = RequestError> + Send> {
    match request.as_ref() {
        constants::CMD_REGISTER_USER => {
            let result = done(args.get_or_request_error(constants::ARG_USER_NAME))
                .join4(
                    done(args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TYPE)),
                    done(args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TOKEN)),
                    ok(args.get_or_empty(constants::ARG_OVERRIDES)),
                )
                .and_then(
                    move |(user_name, social_network_type, social_network_token, overrides)| {
                        client_cmd::register_user(
                            user_name,
                            social_network_type,
                            social_network_token,
                            overrides,
                            config,
                            connection,
                            http_client,
                        )
                        .map_err(|err| err.into())
                    },
                )
                .map(|result| {
                    json!({
                        constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
                        constants::FIELD_NAME_USER_ID: result.uid.to_string(),
                        constants::FIELD_NAME_CLIENT_TOKEN: result.client_token.to_string(),
                    })
                })
                .map_err(|err| err.into());
            Box::new(result)
        }
        &_ => Box::new(err(RequestError::new(
            constants::FIELD_STATUS_UNKNOWN_REQUEST,
            &format!("Unknown request: {}", request),
        ))),
    }
}

impl RequestsHandler for RequestsHandlerImpl {
    fn handle(
        &mut self,
        request: String,
        query: String,
    ) -> Box<dyn Future<Item = String, Error = ()> + Send> {
        let result = self
            .handle_impl(request, query)
            .map(|response| response.to_string())
            .or_else(|error| {
                let response = json!({
                    constants::FIELD_NAME_STATUS: error.status,
                    constants::FIELD_NAME_ERROR_DESCRIPTION: error.error_description
                });
                ok(response.to_string())
            });
        Box::new(result)
    }
}

fn query_to_args(query: String) -> Result<HashMap<String, String>, RequestError> {
    let mut result = HashMap::new();
    if query.is_empty() {
        return Ok(result);
    }

    let pairs = query.split("&");
    for pair in pairs {
        let mut key_and_value = pair.split("=");
        let key = key_and_value.next();
        let value = key_and_value.next();
        if key_and_value.next().is_some() {
            return Err(RequestError::new(
                constants::FIELD_STATUS_INVALID_QUERY,
                &format!("invalid key-value pair: {}", pair),
            ));
        }
        match (key, value) {
            (Some(key), Some(value)) => {
                result.insert(key.to_string(), value.to_string());
            }
            _ => {
                return Err(RequestError::new(
                    constants::FIELD_STATUS_INVALID_QUERY,
                    &format!("invalid key-value pair: {}", pair),
                ));
            }
        }
    }

    return Ok(result);
}

struct RequestError {
    status: String,
    error_description: String,
}

impl RequestError {
    fn new(status: &str, error_description: &str) -> Self {
        RequestError {
            status: status.to_string(),
            error_description: error_description.to_string(),
        }
    }
}

trait HashMapAdditionalOperations {
    fn get_or_request_error(&self, key: &str) -> Result<String, RequestError>;
    fn get_or_empty(&self, key: &str) -> String;
}
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

impl From<PoolError> for RequestError {
    fn from(error: PoolError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Pool error: {}", error),
        )
    }
}

impl From<ServerError> for RequestError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError(error @ ServerErrorKind::VkUidDuplicationError, _) => RequestError::new(
                constants::FIELD_STATUS_ALREADY_REGISTERED,
                &format!("User already registered: {}", error),
            ),
            ServerError(error @ ServerErrorKind::VKTokenCheckError(_, _), _) => RequestError::new(
                constants::FIELD_STATUS_TOKEN_CHECK_FAIL,
                &format!("Token check fail: {}", error),
            ),
            ServerError(error @ _, _) => RequestError::new(
                constants::FIELD_STATUS_INTERNAL_ERROR,
                &format!("Internal error: {}", error),
            ),
        }
    }
}

impl From<uuid::ParseError> for RequestError {
    fn from(error: uuid::ParseError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INVALID_UUID,
            &format!("Invalid UUID: {}", error),
        )
    }
}

#[cfg(test)]
#[path = "./requests_handler_impl_test.rs"]
mod requests_handler_impl_test;
