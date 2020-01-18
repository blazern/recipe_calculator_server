use futures::done;
use futures::future::ok;
use futures::Future;
use percent_encoding::percent_decode;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use config::Config;
use db::pool::connection_pool::ConnectionPool;
use db::pool::connection_pool::ConnectionType;
use http_client::HttpClient;

use super::cmds::cmds_hub::CmdsHub;
use super::constants;
use super::error::Error;
use super::request_error::RequestError;
use super::requests_handler::RequestsHandler;

pub struct RequestsHandlerImpl {
    connection_pool: ConnectionPool,
    config: Config,
    http_client: Arc<HttpClient>,
    cmds_hub: Arc<CmdsHub>,
}

impl RequestsHandlerImpl {
    pub fn new(config: Config) -> Result<RequestsHandlerImpl, Error> {
        Self::new_with_overrides(config, &JsonValue::Null)
    }

    pub fn new_with_overrides(
        config: Config,
        overrides: &JsonValue,
    ) -> Result<RequestsHandlerImpl, Error> {
        let mut pool = ConnectionPool::new(ConnectionType::UserConnection, config.clone());
        let connection = pool.borrow()?;

        Ok(RequestsHandlerImpl {
            connection_pool: pool,
            config,
            http_client: Arc::new(HttpClient::new()?),
            cmds_hub: Arc::new(CmdsHub::new(overrides, connection)?),
        })
    }

    fn handle_impl(
        &mut self,
        request: String,
        query: String,
    ) -> impl Future<Item = JsonValue, Error = RequestError> + Send {
        let connection = self.connection_pool.borrow();
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let cmds_hub = self.cmds_hub.clone();

        done(connection)
            .map_err(|err| err.into())
            .map(|connection| (request, query, connection, cmds_hub))
            .and_then(|(request, query, connection, cmds_hub)| {
                done(query_to_args(query)).map(|args| (request, args, connection, cmds_hub))
            })
            .and_then(|(request, args, connection, cmds_hub)| {
                cmds_hub.handle(request, args, connection, config, http_client)
            })
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
                    constants::FIELD_NAME_STATUS: error.status(),
                    constants::FIELD_NAME_ERROR_DESCRIPTION: error.error_description()
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

    let pairs = query.split('&');
    for pair in pairs {
        let mut key_and_value = pair.split('=');
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
                let key = decode_query_part(key)?;
                let value = decode_query_part(value)?;
                result.insert(key, value);
            }
            _ => {
                return Err(RequestError::new(
                    constants::FIELD_STATUS_INVALID_QUERY,
                    &format!("invalid key-value pair: {}", pair),
                ));
            }
        }
    }

    Ok(result)
}

fn decode_query_part(part: &str) -> Result<String, RequestError> {
    let result = percent_decode(part.as_bytes()).decode_utf8();
    match result {
        Ok(result) => Ok(result.to_string()),
        Err(err) => Err(RequestError::new(
            constants::FIELD_STATUS_INVALID_QUERY,
            &format!("could not URL decode query part: {}, err: {}", part, err),
        )),
    }
}
