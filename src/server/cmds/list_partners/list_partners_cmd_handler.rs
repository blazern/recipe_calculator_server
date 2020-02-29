use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;
use crate::db::core::app_user;
use crate::db::core::paired_partners;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;

use crate::db::core::paired_partners::PairingState;
use crate::server::cmds::cmd_handler::{CmdHandleResult, CmdHandleResultFuture, CmdHandler};
use crate::server::cmds::utils::extract_user_from_query_args;
use crate::server::constants;

#[derive(Default)]
pub struct ListPartnersCmdHandler;

impl CmdHandler for ListPartnersCmdHandler {
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

impl ListPartnersCmdHandler {
    pub fn new() -> Self {
        ListPartnersCmdHandler::default()
    }

    async fn handle_impl(
        args: HashMap<String, String>,
        mut connections_pool: ConnectionPool,
        _config: Config,
        _http_client: Arc<HttpClient>,
    ) -> CmdHandleResult {
        let connection = connections_pool.borrow_connection()?;
        let user = extract_user_from_query_args(&args, &connection)?;
        let partners_pairs = paired_partners::select_by_partner_user_id_and_state(
            user.id(),
            PairingState::Done,
            &connection,
        )?;

        let partners_ids: Vec<i32> = partners_pairs
            .iter()
            .map(|item| {
                if item.partner1_user_id() == user.id() {
                    item.partner2_user_id()
                } else {
                    item.partner1_user_id()
                }
            })
            .collect();

        let mut json_partners = Vec::new();
        for id in partners_ids {
            let partner = app_user::select_by_id(id, &connection)?;
            let partner = match partner {
                Some(partner) => partner,
                None => continue, // Partner was deleted a couple of ms ago
            };
            json_partners.push(json!({
                constants::FIELD_NAME_PARTNER_USER_ID: partner.uid().to_string(),
                constants::FIELD_NAME_PARTNER_NAME: partner.name()
            }))
        }

        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
            constants::FIELD_NAME_PARTNERS: json_partners
        }))
    }
}

#[cfg(test)]
#[path = "./list_partners_cmd_handler_test.rs"]
mod list_partners_cmd_handler_test;
