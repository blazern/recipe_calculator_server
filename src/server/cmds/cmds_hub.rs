use futures::future::err;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::db::pool::connection_pool::{BorrowedDBConnection, ConnectionPool};
use crate::outside::http_client::HttpClient;
use crate::server::cmds::cmd_handler::CmdHandleResultFuture;
use crate::server::constants;
use crate::server::error::Error;
use crate::server::request_error::RequestError;

use super::cmd_handler::CmdHandler;
use super::direct_partner_msg::direct_partner_msg_cmd_handler::DirectPartnerMsgCmdHandler;
use super::list_partners::list_partners_cmd_handler::ListPartnersCmdHandler;
use super::move_device_account::move_device_account_cmd_handler::MoveDeviceAccountCmdHandler;
use super::pairing_request::pairing_request_cmd_handler::PairingRequestCmdHandler;
use super::register_user::register_user_cmd_handler::RegisterUserCmdHandler;
use super::start_pairing::start_pairing_cmd_handler::StartPairingCmdHandler;
use super::unpair::unpair_cmd_handler::UnpairCmdHandler;
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
        cmd_handlers.insert(
            constants::CMD_PAIRING_REQUEST,
            Box::new(PairingRequestCmdHandler::new(overrides)),
        );
        cmd_handlers.insert(
            constants::CMD_MOVE_DEVICE_ACCOUNT,
            Box::new(MoveDeviceAccountCmdHandler::new()),
        );
        cmd_handlers.insert(
            constants::CMD_LIST_PARTNERS,
            Box::new(ListPartnersCmdHandler::new()),
        );
        cmd_handlers.insert(constants::CMD_UNPAIR, Box::new(UnpairCmdHandler::new()));
        cmd_handlers.insert(
            constants::CMD_DIRECT_PARTNER_MSG,
            Box::new(DirectPartnerMsgCmdHandler::new(overrides)),
        );
        Ok(CmdsHub { cmd_handlers })
    }

    pub fn handle(
        &self,
        request: String,
        args: HashMap<String, String>,
        body: String,
        connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResultFuture {
        let handler = self.cmd_handlers.get(request.as_str());
        if let Some(handler) = handler {
            handler.handle(args, body, connections_pool, config, http_client)
        } else {
            Box::pin(err(RequestError::new(
                constants::FIELD_STATUS_UNKNOWN_REQUEST.to_owned(),
                format!("Unknown request: {}", request),
            )))
        }
    }
}
