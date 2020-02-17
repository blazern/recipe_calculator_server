use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::db::core::app_user::AppUser;
use crate::db::core::{app_user, gp_user, vk_user};
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;
use crate::server::cmds::cmd_handler::{CmdHandleResult, CmdHandleResultFuture, CmdHandler};
use crate::server::cmds::register_user::social_network_token_check::{
    check_token, TokenCheckSuccess,
};
use crate::server::cmds::utils::HashMapAdditionalOperations;
use crate::server::constants;
use crate::server::request_error::RequestError;
use uuid::Uuid;

#[derive(Default)]
pub struct MoveDeviceAccountCmdHandler {}

impl CmdHandler for MoveDeviceAccountCmdHandler {
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

impl MoveDeviceAccountCmdHandler {
    pub fn new() -> MoveDeviceAccountCmdHandler {
        MoveDeviceAccountCmdHandler::default()
    }

    async fn handle_impl(
        args: HashMap<String, String>,
        mut connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResult {
        let connection = connections_pool.borrow_connection()?;
        let social_network_type = args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TYPE)?;
        let social_network_token =
            args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TOKEN)?;
        let overrides = args.get_or_empty(constants::ARG_OVERRIDES);

        let token_check_result = check_token(
            social_network_type,
            social_network_token,
            &overrides,
            http_client,
            config,
        )
        .await?;

        let db_connection_ref = &connection;

        let user = match token_check_result {
            TokenCheckSuccess::VK { uid } => {
                let vk_user = vk_user::select_by_vk_uid(&uid, db_connection_ref)?;
                let vk_user = match vk_user {
                    Some(vk_user) => vk_user,
                    None => {
                        return Err(RequestError::new(
                            constants::FIELD_STATUS_USER_NOT_FOUND,
                            "User with given social network token not found",
                        ))
                    }
                };
                app_user::select_by_id(vk_user.app_user_id(), db_connection_ref)?
            }
            TokenCheckSuccess::GP { uid } => {
                let gp_user = gp_user::select_by_gp_uid(&uid, db_connection_ref)?;
                let gp_user = match gp_user {
                    Some(gp_user) => gp_user,
                    None => {
                        return Err(RequestError::new(
                            constants::FIELD_STATUS_USER_NOT_FOUND,
                            "User with given social network token not found",
                        ))
                    }
                };
                app_user::select_by_id(gp_user.app_user_id(), db_connection_ref)?
            }
        };
        let user = extract_possibly_deleted_user(user)?;

        let new_client_token = Uuid::new_v4();
        let user = app_user::update_client_token(user, &new_client_token, db_connection_ref)?;
        let user = extract_possibly_deleted_user(user)?;

        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
            constants::FIELD_NAME_USER_ID: user.uid().to_string(),
            constants::FIELD_NAME_CLIENT_TOKEN: user.client_token().to_string(),
            constants::FIELD_NAME_USER_NAME: user.name(),
        }))
    }
}

fn extract_possibly_deleted_user(user: Option<AppUser>) -> Result<AppUser, RequestError> {
    // Low probability, but user could be deleted between gp/vk user selection (or other operation)
    // and this moment.
    match user {
        Some(user) => Ok(user),
        None => Err(RequestError::new(
            constants::FIELD_STATUS_USER_NOT_FOUND,
            "User with given social network token not found",
        )),
    }
}

#[cfg(test)]
#[path = "./move_device_account_cmd_handler_test.rs"]
mod move_device_account_cmd_handler_test;
