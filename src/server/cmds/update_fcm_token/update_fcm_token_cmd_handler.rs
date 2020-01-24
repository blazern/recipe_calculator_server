use std::collections::HashMap;

use futures::done;
use futures::Future;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use config::Config;
use db::core::fcm_token;
use db::core::transaction;
use db::pool::connection_pool::BorrowedDBConnection;
use http_client::HttpClient;

use server::cmds::cmd_handler::CmdHandler;
use server::cmds::utils::extract_user_from_query_args;
use server::cmds::utils::HashMapAdditionalOperations;
use server::constants;
use server::request_error::RequestError;

pub struct UpdateFcmTokenCmdHandler;

impl CmdHandler for UpdateFcmTokenCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> Box<dyn Future<Item = JsonValue, Error = RequestError> + Send> {
        Box::new(done(self.handle_impl(
            args,
            connection,
            config,
            http_client,
        )))
    }
}

impl UpdateFcmTokenCmdHandler {
    pub fn new() -> Self {
        UpdateFcmTokenCmdHandler {}
    }

    fn handle_impl(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        _config: Config,
        _http_client: Arc<HttpClient>,
    ) -> Result<JsonValue, RequestError> {
        let user = extract_user_from_query_args(&args, &connection)?;
        let fcm_token_value = args.get_or_request_error(constants::ARG_FCM_TOKEN)?;
        transaction::start::<(), RequestError, _>(&connection, || {
            fcm_token::delete_by_user_id(user.id(), &connection)?;
            fcm_token::insert(fcm_token::new(fcm_token_value, &user), &connection)?;
            Ok(())
        })?;
        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
        }))
    }
}

#[cfg(test)]
#[path = "./update_fcm_token_cmd_handler_test.rs"]
mod update_fcm_token_cmd_handler_test;
