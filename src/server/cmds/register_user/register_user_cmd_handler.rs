use futures::done;
use futures::future::ok;
use futures::Future;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use config::Config;
use db::pool::connection_pool::BorrowedDBConnection;
use http_client::HttpClient;
use server::constants;
use server::request_error::RequestError;

use server::cmds::cmd_handler::CmdHandler;
use server::cmds::hash_map_additional_operations::HashMapAdditionalOperations;

use super::register_user_impl;

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
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> Box<dyn Future<Item = Value, Error = RequestError> + Send> {
        let result = done(args.get_or_request_error(constants::ARG_USER_NAME))
            .join4(
                done(args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TYPE)),
                done(args.get_or_request_error(constants::ARG_SOCIAL_NETWORK_TOKEN)),
                ok(args.get_or_empty(constants::ARG_OVERRIDES)),
            )
            .and_then(
                move |(user_name, social_network_type, social_network_token, overrides)| {
                    register_user_impl::register_user(
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
            .map_err(|err| err);
        Box::new(result)
    }
}

#[cfg(test)]
#[path = "./register_user_cmd_handler_test.rs"]
mod register_user_cmd_handler_test;
