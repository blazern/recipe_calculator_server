use std::collections::HashMap;

use std::sync::Arc;

use crate::config::Config;
use crate::db::core::app_user;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;

use crate::server::cmds::cmd_handler::{CmdHandleResult, CmdHandleResultFuture, CmdHandler};
use crate::server::cmds::utils::db_transaction;
use crate::server::cmds::utils::extract_user_from_query_args;
use crate::server::cmds::utils::HashMapAdditionalOperations;
use crate::server::constants;

#[derive(Default)]
pub struct UpdateUserNameCmdHandler;

impl CmdHandler for UpdateUserNameCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        _body: String,
        connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResultFuture {
        Box::pin(Self::handle_impl(
            args,
            connections_pool,
            config,
            http_client,
        ))
    }
}

impl UpdateUserNameCmdHandler {
    pub fn new() -> Self {
        UpdateUserNameCmdHandler::default()
    }

    async fn handle_impl(
        args: HashMap<String, String>,
        mut connections_pool: ConnectionPool,
        _config: Config,
        _http_client: Arc<HttpClient>,
    ) -> CmdHandleResult {
        let connection = connections_pool.borrow_connection()?;
        let user = extract_user_from_query_args(&args, &connection)?;
        let user_name_value = args.get_or_request_error(constants::ARG_USER_NAME)?;
        db_transaction(&connection, || {
            app_user::update_user_name(user, &user_name_value, &connection)?;
            Ok(())
        })?;
        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
        }))
    }
}

#[cfg(test)]
#[path = "./update_user_name_cmd_handler_test.rs"]
mod update_user_name_cmd_handler_test;
