use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;
use crate::server::constants;
use crate::server::request_error::RequestError;

use crate::server::cmds::cmd_handler::CmdHandleResultFuture;
use crate::server::cmds::cmd_handler::CmdHandler;
use crate::server::cmds::utils::HashMapAdditionalOperations;

use super::register_user_impl;

#[derive(Default)]
pub struct RegisterUserCmdHandler {}
impl RegisterUserCmdHandler {
    pub fn new() -> RegisterUserCmdHandler {
        RegisterUserCmdHandler {}
    }
}

impl CmdHandler for RegisterUserCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResultFuture {
        Box::pin(handle_impl(args, connections_pool, config, http_client))
    }
}

async fn handle_impl(
    args: HashMap<String, String>,
    mut connections_pool: ConnectionPool,
    config: Config,
    http_client: Arc<HttpClient>,
) -> Result<JsonValue, RequestError> {
    let connection = connections_pool.borrow_connection()?;
    let user_name = args.get_or_request_error(constants::ARG_USER_NAME)?;
    let social_network_type = args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TYPE)?;
    let social_network_token = args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TOKEN)?;
    let overrides = args.get_or_empty(constants::ARG_OVERRIDES);

    let result = register_user_impl::register_user(
        user_name,
        social_network_type,
        social_network_token,
        overrides,
        config,
        connection,
        http_client,
    )
    .await?;

    let result = json!({
        constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
        constants::FIELD_NAME_USER_ID: result.uid.to_string(),
        constants::FIELD_NAME_CLIENT_TOKEN: result.client_token.to_string(),
    });

    Ok(result)
}

#[cfg(test)]
#[path = "./register_user_cmd_handler_test.rs"]
mod register_user_cmd_handler_test;
