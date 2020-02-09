use percent_encoding::percent_decode;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::config::Config;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::db::pool::connection_pool::ConnectionType;
use crate::outside::http_client::HttpClient;
use crate::server::cmds::cmd_handler::CmdHandleResult;

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
        let connection = pool.borrow_connection()?;

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
    ) -> impl Future<Output = CmdHandleResult> {
        let pool = self.connection_pool.clone();
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let cmds_hub = self.cmds_hub.clone();

        async move {
            let args = query_to_args(query)?;
            cmds_hub
                .handle(request, args, pool, config, http_client)
                .await
        }
    }
}

impl RequestsHandler for RequestsHandlerImpl {
    fn handle(
        &mut self,
        request: String,
        query: String,
        _headers: HashMap<String, String>,
        _body: String,
    ) -> Pin<Box<dyn Future<Output = String> + Send>> {
        let response = self.handle_impl(request, query);
        let result = async {
            let response = response.await;
            match response {
                Ok(response) => response.to_string(),
                Err(error) => {
                    let response = json!({
                        constants::FIELD_NAME_STATUS: error.status(),
                        constants::FIELD_NAME_ERROR_DESCRIPTION: error.error_description()
                    });
                    response.to_string()
                }
            }
        };
        Box::pin(result)
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
