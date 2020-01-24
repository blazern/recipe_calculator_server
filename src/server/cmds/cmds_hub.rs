use futures::future::err;
use futures::Future;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use config::Config;
use db::pool::connection_pool::BorrowedDBConnection;
use http_client::HttpClient;
use server::error::Error;

use server::constants;
use server::request_error::RequestError;

use super::cmd_handler::CmdHandler;
use super::register_user::register_user_cmd_handler::RegisterUserCmdHandler;
use super::start_pairing::start_pairing_cmd_handler::StartPairingCmdHandler;
use super::update_fcm_token::update_fcm_token_cmd_handler::UpdateFcmTokenCmdHandler;

type CmdsHashMap = HashMap<&'static str, Box<dyn CmdHandler + Send + Sync>>;

pub struct CmdsHub {
    cmd_handlers: CmdsHashMap,
}

impl CmdsHub {
    pub fn new(overrides: &JsonValue, connection: BorrowedDBConnection) -> Result<CmdsHub, Error> {
        let mut cmd_handlers = CmdsHashMap::new();
        cmd_handlers.insert(
            constants::CMD_REGISTER_USER,
            Box::new(RegisterUserCmdHandler::new()),
        );
        cmd_handlers.insert(
            constants::CMD_START_PAIRING,
            Box::new(StartPairingCmdHandler::new(overrides, &connection)?),
        );
        cmd_handlers.insert(
            constants::CMD_UPDATE_FCM_TOKEN,
            Box::new(UpdateFcmTokenCmdHandler::new()),
        );
        Ok(CmdsHub { cmd_handlers })
    }

    pub fn handle(
        &self,
        request: String,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> Box<dyn Future<Item = JsonValue, Error = RequestError> + Send> {
        let handler = self.cmd_handlers.get(request.as_str());
        if let Some(handler) = handler {
            handler.handle(args, connection, config, http_client)
        } else {
            Box::new(err(RequestError::new(
                constants::FIELD_STATUS_UNKNOWN_REQUEST,
                &format!("Unknown request: {}", request),
            )))
        }
    }
}
