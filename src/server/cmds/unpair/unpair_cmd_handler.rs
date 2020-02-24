use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::db::core::app_user;
use crate::db::core::paired_partners;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;

use crate::server::cmds::cmd_handler::{CmdHandleResult, CmdHandleResultFuture, CmdHandler};
use crate::server::cmds::utils::extract_user_from_query_args;
use crate::server::cmds::utils::HashMapAdditionalOperations;
use crate::server::constants;

#[derive(Default)]
pub struct UnpairCmdHandler;

impl CmdHandler for UnpairCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
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

impl UnpairCmdHandler {
    pub fn new() -> Self {
        UnpairCmdHandler::default()
    }

    async fn handle_impl(
        args: HashMap<String, String>,
        mut connections_pool: ConnectionPool,
        _config: Config,
        _http_client: Arc<HttpClient>,
    ) -> CmdHandleResult {
        let connection = connections_pool.borrow_connection()?;
        let user = extract_user_from_query_args(&args, &connection)?;
        let partner_uid = args.get_or_request_error(constants::ARG_PARTNER_USER_ID)?;

        let partner = app_user::select_by_uid(&Uuid::from_str(&partner_uid)?, &connection)?;
        if let Some(partner) = partner {
            paired_partners::delete_by_partners_user_ids(user.id(), partner.id(), &connection)?;
        }
        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
        }))
    }
}

#[cfg(test)]
#[path = "./unpair_cmd_handler_test.rs"]
mod unpair_cmd_handler_test;
