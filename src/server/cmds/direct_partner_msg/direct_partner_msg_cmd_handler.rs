use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::db::core::app_user;
use crate::db::core::paired_partners;
use crate::db::pool::connection_pool::ConnectionPool;
use crate::outside::http_client::HttpClient;

use crate::outside::fcm::FCM_ADDR;
use crate::server::cmds::cmd_handler::{CmdHandleResult, CmdHandleResultFuture, CmdHandler};
use crate::server::cmds::utils::extract_user_from_query_args;
use crate::server::cmds::utils::notify_user;
use crate::server::cmds::utils::HashMapAdditionalOperations;
use crate::server::constants;
use crate::server::request_error::RequestError;

pub struct DirectPartnerMsgCmdHandler {
    fcm_address: String,
}

impl CmdHandler for DirectPartnerMsgCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        body: String,
        connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResultFuture {
        Box::pin(Self::handle_impl(
            args,
            body,
            connections_pool,
            config,
            http_client,
            self.fcm_address.clone(),
        ))
    }
}

impl DirectPartnerMsgCmdHandler {
    pub fn new(overrides: &JsonValue) -> Self {
        let args = get_construction_args(overrides);
        DirectPartnerMsgCmdHandler {
            fcm_address: args.fcm_address,
        }
    }

    async fn handle_impl(
        args: HashMap<String, String>,
        body: String,
        mut connections_pool: ConnectionPool,
        config: Config,
        http_client: Arc<HttpClient>,
        fcm_address: String,
    ) -> CmdHandleResult {
        let connection = connections_pool.borrow_connection()?;
        let user = extract_user_from_query_args(&args, &connection)?;
        let partner_uid = args.get_or_request_error(constants::ARG_PARTNER_USER_ID)?;
        let partner = app_user::select_by_uid(&Uuid::from_str(&partner_uid)?, &connection)?;
        let partner = match partner {
            Some(partner) => partner,
            None => {
                return Err(RequestError::new(
                    constants::FIELD_STATUS_PARTNER_USER_NOT_FOUND.to_string(),
                    format!("Partner user was not found. Given uid: {:?}", partner_uid),
                ))
            }
        };

        let pp1 =
            paired_partners::select_by_partners_user_ids(user.id(), partner.id(), &connection)?;
        let pp2 =
            paired_partners::select_by_partners_user_ids(partner.id(), user.id(), &connection)?;
        if pp1.is_none() && pp2.is_none() {
            return Err(RequestError::new(
                constants::FIELD_STATUS_PARTNER_USER_NOT_FOUND.to_string(),
                format!("Partner user was not found. Given uid: {:?}", partner_uid),
            ));
        }

        let json = json!({
            constants::SERV_FIELD_MSG_TYPE: constants::SERV_MSG_DIRECT_MSG_FROM_PARTNER,
            constants::SERV_FIELD_PARTNER_USER_ID: user.uid(),
            constants::SERV_FIELD_PARTNER_NAME: user.name(),
            constants::SERV_FIELD_MSG: body,
        });
        // NOTE: we don't use the '?' operator on the send result - we want to respond
        // with OK status to our client even if notifications sending will fail
        let _notif_res = notify_user(
            &partner,
            json.to_string(),
            connections_pool,
            &config,
            &fcm_address,
            http_client,
        )
        .await;

        Ok(json!({
            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
        }))
    }
}

#[cfg(test)]
pub fn insert_construction_overrides(overrides: &mut JsonValue, fcm_address: String) {
    let overrides = overrides
        .as_object_mut()
        .expect("Can insert only into object");
    overrides.insert("direct_partner_msg_overrides".to_owned(), json!({}));
    let overrides = overrides["direct_partner_msg_overrides"]
        .as_object_mut()
        .unwrap();
    overrides.insert("fcm_address_override".to_owned(), json!(fcm_address));
}

struct ConstructionArgs {
    fcm_address: String,
}

fn get_construction_args(overrides: &JsonValue) -> ConstructionArgs {
    if let Some(overrides) = extract_construction_overrides(overrides) {
        overrides
    } else {
        ConstructionArgs {
            fcm_address: FCM_ADDR.to_owned(),
        }
    }
}

#[cfg(not(test))]
fn extract_construction_overrides(_overrides: &JsonValue) -> Option<ConstructionArgs> {
    None
}

#[cfg(test)]
fn extract_construction_overrides(overrides: &JsonValue) -> Option<ConstructionArgs> {
    match &overrides["direct_partner_msg_overrides"].as_object() {
        Some(overrides) => Some(ConstructionArgs {
            fcm_address: overrides["fcm_address_override"]
                .as_str()
                .unwrap()
                .to_owned(),
        }),
        None => None,
    }
}

#[cfg(test)]
#[path = "./direct_partner_msg_cmd_handler_test.rs"]
mod direct_partner_msg_cmd_handler_test;
