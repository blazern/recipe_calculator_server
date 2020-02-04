use futures::future::err;
use futures::future::ok;
use futures::future::Either;
use futures::Future;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;

use crate::db::core::app_user;
use crate::db::core::app_user::AppUser;
use crate::db::core::fcm_token;
use crate::db::core::paired_partners;
use crate::db::core::paired_partners::PairingState;
use crate::db::core::taken_pairing_code;
use crate::db::pool::connection_pool::BorrowedDBConnection;

use crate::outside::fcm;
use crate::outside::http_client::HttpClient;

use crate::server::cmds::cmd_handler::CmdHandleResult;
use crate::server::cmds::cmd_handler::CmdHandler;
use crate::server::cmds::utils::db_transaction;
use crate::server::cmds::utils::extract_user_from_query_args;
use crate::server::constants;
use crate::server::request_error::RequestError;

use crate::utils::now_source::{DefaultNowSource, NowSource};

pub const PAIRING_CONFIRMATION_EXPIRATION_DELAY_SECS: i64 = 60 * 60 * 24;

pub struct PairingRequestCmdHandler {
    family: String,
    now_source: DefaultNowSource,
}

impl CmdHandler for PairingRequestCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> CmdHandleResult {
        let res = self.handle_impl(args, connection, config, http_client);
        match res {
            Ok(future) => future,
            Err(request_err) => Box::new(err(request_err)),
        }
    }
}

impl PairingRequestCmdHandler {
    pub fn new(overrides: &JsonValue) -> Self {
        let args = get_construction_args(overrides);
        PairingRequestCmdHandler {
            family: args.family,
            now_source: DefaultNowSource::default(),
        }
    }

    /// Returns Result<Future<..>>, which means that there
    /// can be an instant success/failure, or a delayed one.
    fn handle_impl(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        config: Config,
        http_client: Arc<HttpClient>,
    ) -> Result<CmdHandleResult, RequestError> {
        let now = match extract_now_override(&args) {
            Some(now) => now,
            None => self.now_source.now_secs()?,
        };
        let fcm_address = match extract_cmd_fcm_address_override(&args) {
            Some(fcm_address) => fcm_address,
            None => fcm::FCM_ADDR.to_owned(),
        };

        // cleanup
        paired_partners::delete_with_state_and_older_than(
            PairingState::NotConfirmed,
            now - PAIRING_CONFIRMATION_EXPIRATION_DELAY_SECS,
            &connection,
        )?;

        let user = extract_user_from_query_args(&args, &connection)?;
        let partner_user = extract_partner_user(
            &self.family,
            args.get(constants::ARG_PARTNER_PAIRING_CODE),
            args.get(constants::ARG_PARTNER_USER_ID),
            &connection,
        )?;

        let pp1 = paired_partners::select_by_partners_user_ids(
            user.id(),
            partner_user.id(),
            &connection,
        )?;
        let pp2 = paired_partners::select_by_partners_user_ids(
            partner_user.id(),
            user.id(),
            &connection,
        )?;
        if is_pairing_finished(&pp1) || is_pairing_finished(&pp2) {
            // Already paired!
            let send_fut = notify_of_pairing_finish(
                &user,
                &partner_user,
                &connection,
                &config,
                &fcm_address,
                http_client.clone(),
            );
            // NOTE: |then| is used instead of |and_then| - we'll send status OK even if
            // notifications will fail.
            let res = send_fut.then(|_| {
                // TODO: log if send finished with error
                Ok(json!({
                    constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
                }))
            });
            return Ok(Box::new(res));
        }

        // Confirm pairing, if partner2 already started it
        if let Some(pp2) = pp2 {
            db_transaction(&connection, || {
                // Partner2 already sent a pairing request to Partner1
                let time = pp2.pairing_start_time();
                paired_partners::delete_by_id(pp2.id(), &connection)?;
                let pp2 = paired_partners::new(&partner_user, &user, PairingState::Done, time);
                paired_partners::insert(pp2, &connection)?;
                Ok(())
            })?;

            let send_fut1 = notify_of_pairing_finish(
                &user,
                &partner_user,
                &connection,
                &config,
                &fcm_address,
                http_client.clone(),
            );
            let send_fut2 = notify_of_pairing_finish(
                &partner_user,
                &user,
                &connection,
                &config,
                &fcm_address,
                http_client.clone(),
            );

            // NOTE: |then| is used instead of |and_then| - we'll send status OK even if
            // notifications will fail.
            let res = send_fut1.join(send_fut2).then(|_| {
                // TODO: log if send finished with error
                Ok(json!({
                    constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
                }))
            });
            return Ok(Box::new(res));
        }

        // Send pairing request
        db_transaction(&connection, || {
            // Delete an unfinished pairing, if it exists.
            if let Some(pp1) = pp1 {
                paired_partners::delete_by_id(pp1.id(), &connection)?;
            }
            let pp = paired_partners::new(&user, &partner_user, PairingState::NotConfirmed, now);
            paired_partners::insert(pp, &connection)?;
            Ok(())
        })?;

        let send_fut = notify_of_pairing_request(
            &partner_user,
            &user,
            now,
            &connection,
            &config,
            &fcm_address,
            http_client.clone(),
        );
        // NOTE: |then| is used instead of |and_then| - we'll send status OK even if
        // notifications will fail.
        let res = send_fut.then(|_| {
            // TODO: log if send finished with error
            Ok(json!({
                constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK
            }))
        });
        Ok(Box::new(res))
    }
}

fn extract_partner_user(
    family: &str,
    partner_pairing_code: Option<&String>,
    partner_uid: Option<&String>,
    connection: &BorrowedDBConnection,
) -> Result<AppUser, RequestError> {
    if partner_uid.is_none() && partner_pairing_code.is_none() {
        return Err(RequestError::new(
            constants::FIELD_STATUS_PARAM_MISSING,
            "Need either partner user ID or partner pairng code, none provided",
        ));
    }

    if let Some(partner_pairing_code) = partner_pairing_code {
        let partner_pairing_code = match partner_pairing_code.parse::<i32>() {
            Ok(partner_pairing_code) => partner_pairing_code,
            Err(error) => {
                return Err(RequestError::new(
                    constants::FIELD_STATUS_INVALID_PARTNER_PAIRING_CODE,
                    &format!(
                        "Partner code is invalid: {}, err: {}",
                        partner_pairing_code, error
                    ),
                ))
            }
        };
        let partner_pairing_code =
            taken_pairing_code::select_by_value(partner_pairing_code, family, connection)?;
        if let Some(partner_pairing_code) = partner_pairing_code {
            let user_id = partner_pairing_code.app_user_id();
            let user = app_user::select_by_id(user_id, connection)?;
            if let Some(user) = user {
                return Ok(user);
            }
        }
    }

    if let Some(partner_uid) = partner_uid {
        let partner_uid = Uuid::from_str(partner_uid)?;
        let user = app_user::select_by_uid(&partner_uid, connection)?;
        if let Some(user) = user {
            return Ok(user);
        }
    }

    Err(RequestError::new(
        constants::FIELD_STATUS_PARTNER_USER_NOT_FOUND,
        &format!(
            "Partner user was not found. Given code: {:?}, uid: {:?}",
            partner_pairing_code, partner_uid
        ),
    ))
}

fn is_pairing_finished(pp: &Option<paired_partners::PairedPartners>) -> bool {
    if let Some(pp) = pp {
        pp.pairing_state() == PairingState::Done
    } else {
        false
    }
}

fn notify_of_pairing_finish(
    user: &AppUser,
    paired_partner: &AppUser,
    connection: &BorrowedDBConnection,
    config: &Config,
    fcm_address: &str,
    http_client: Arc<HttpClient>,
) -> Box<dyn Future<Item = (), Error = RequestError> + Send> {
    let json = json!({
        constants::SERV_FIELD_MSG_TYPE: constants::SERV_MSG_PAIRED_WITH_PARTNER,
        constants::SERV_FIELD_PAIRING_PARTNER_USER_ID: paired_partner.uid(),
        constants::SERV_FIELD_PARTNER_NAME: paired_partner.name()
    });
    notify_user(user, &json, connection, config, fcm_address, http_client)
}

fn notify_of_pairing_request(
    user: &AppUser,
    paired_partner: &AppUser,
    now: i64,
    connection: &BorrowedDBConnection,
    config: &Config,
    fcm_address: &str,
    http_client: Arc<HttpClient>,
) -> Box<dyn Future<Item = (), Error = RequestError> + Send> {
    let expiration_date = now + PAIRING_CONFIRMATION_EXPIRATION_DELAY_SECS;
    let json = json!({
        constants::SERV_FIELD_MSG_TYPE: constants::SERV_MSG_PAIRING_REQUEST_FROM_PARTNER,
        constants::SERV_FIELD_PAIRING_PARTNER_USER_ID: paired_partner.uid(),
        constants::SERV_FIELD_PARTNER_NAME: paired_partner.name(),
        constants::SERV_FIELD_REQUEST_EXPIRATION_DATE: expiration_date
    });
    notify_user(user, &json, connection, config, fcm_address, http_client)
}

/// Returns future which resolves when a notification is sent to the |user|.
/// Or immediately if the user doesn't have a FCM-token.
fn notify_user(
    user: &AppUser,
    json: &JsonValue,
    connection: &BorrowedDBConnection,
    config: &Config,
    fcm_address: &str,
    http_client: Arc<HttpClient>,
) -> Box<dyn Future<Item = (), Error = RequestError> + Send> {
    let fcm_token = fcm_token::select_by_user_id(user.id(), connection);
    let fcm_token = match fcm_token {
        Ok(fcm_token) => fcm_token,
        Err(error) => return Box::new(err(error.into())),
    };

    let fcm_token = if let Some(fcm_token) = fcm_token {
        fcm_token
    } else {
        return Box::new(ok(()));
    };

    // TODO: log errors from send
    let send_res = fcm::send(
        &json,
        fcm_token.token_value(),
        config.fcm_server_token(),
        fcm_address,
        http_client,
    )
    .map_err(|err| err.into())
    .and_then(|send_res| {
        if let fcm::SendResult::Error(error) = send_res {
            Either::A(err(RequestError::new(
                constants::FIELD_STATUS_INTERNAL_ERROR,
                &format!("FCM error: {}", error),
            )))
        } else {
            Either::B(ok(()))
        }
    });
    Box::new(send_res)
}

#[cfg(test)]
pub fn insert_pairing_request_overrides(overrides: &mut JsonValue, family_name: String) {
    let overrides = overrides
        .as_object_mut()
        .expect("Can insert only into object");
    overrides.insert("pairing_request_overrides".to_owned(), json!({}));
    let overrides = overrides["pairing_request_overrides"]
        .as_object_mut()
        .unwrap();
    overrides.insert("family".to_owned(), json!(family_name));
}

struct ConstructionArgs {
    family: String,
}

fn get_construction_args(overrides: &JsonValue) -> ConstructionArgs {
    if let Some(overrides) = extract_construction_overrides(overrides) {
        overrides
    } else {
        ConstructionArgs {
            family: constants::PAIRING_CODES_FAMILY_NAME.to_owned(),
        }
    }
}

#[cfg(not(test))]
fn extract_construction_overrides(_overrides: &JsonValue) -> Option<ConstructionArgs> {
    None
}

#[cfg(test)]
fn extract_construction_overrides(overrides: &JsonValue) -> Option<ConstructionArgs> {
    match &overrides["pairing_request_overrides"].as_object() {
        Some(overrides) => Some(ConstructionArgs {
            family: overrides["family"].as_str().unwrap().to_owned(),
        }),
        None => None,
    }
}

#[cfg(test)]
pub fn insert_cmd_now_override(overrides: &mut JsonValue, now: i64) {
    overrides["now_override"] = json!(now);
}

#[cfg(not(test))]
fn extract_now_override(_args: &HashMap<String, String>) -> Option<i64> {
    None
}

#[cfg(test)]
fn extract_now_override(args: &HashMap<String, String>) -> Option<i64> {
    let json = extract_overrides_json(args);
    if !json["now_override"].is_null() {
        Some(json["now_override"].as_i64().unwrap())
    } else {
        None
    }
}

#[cfg(test)]
fn extract_overrides_json(args: &HashMap<String, String>) -> JsonValue {
    let overrides = match args.get(constants::ARG_OVERRIDES) {
        Some(overrides) => overrides,
        None => return json!({}),
    };

    let json = serde_json::from_str(overrides);
    match json {
        Ok(json) => json,
        Err(_error) => {
            if !overrides.is_empty() {
                panic!("Overrides are not empty but are not json: {}", overrides)
            }
            json!({})
        }
    }
}

#[cfg(test)]
pub fn insert_cmd_fcm_address_override(overrides: &mut JsonValue, address: &str) {
    overrides["fcm_address_override"] = json!(address);
}

#[cfg(not(test))]
fn extract_cmd_fcm_address_override(_args: &HashMap<String, String>) -> Option<String> {
    None
}

#[cfg(test)]
fn extract_cmd_fcm_address_override(args: &HashMap<String, String>) -> Option<String> {
    let json = extract_overrides_json(args);
    if !json["fcm_address_override"].is_null() {
        Some(json["fcm_address_override"].as_str().unwrap().to_owned())
    } else {
        None
    }
}

#[cfg(test)]
#[path = "./pairing_request_cmd_handler_test.rs"]
mod pairing_request_cmd_handler_test;
