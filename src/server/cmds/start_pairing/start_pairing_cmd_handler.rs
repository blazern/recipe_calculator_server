use futures::done;
use futures::future::ok;
use futures::Future;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use config::Config;
use db::core::connection::DBConnection;
use db::pool::connection_pool::BorrowedDBConnection;
use http_client::HttpClient;
use pairing::pairing_code_creator;
use pairing::pairing_code_creator::DefaultNowSource;
use pairing::pairing_code_creator::{DefaultPairingCodeCreatorImpl, NowSource, PairingCodeCreator};
use server::cmds::cmd_handler::CmdHandler;
use server::cmds::utils::extract_user_from_query_args;
use server::constants;
use server::error::Error;
use server::request_error::RequestError;

const PAIRING_CODES_FAMILY_NAME: &str = "default";
const PAIRING_CODES_LIFETIME_SECS: i64 = 60 * 12; // 12 minutes
const PAIRING_CODES_LIFETIME_USER_VISIBLE_SECS: i64 = 60 * 10; // 12 minutes

#[cfg(test)]
pub fn insert_pairing_code_gen_family_override(overrides: &mut JsonValue, family_name: String) {
    insert_pairing_code_gen_extended_override(
        overrides,
        family_name,
        0,
        9999,
        PAIRING_CODES_LIFETIME_SECS,
        false,
    )
}

#[cfg(test)]
pub fn insert_pairing_code_gen_extended_override(
    overrides: &mut JsonValue,
    family_name: String,
    codes_range_left: i32,
    codes_range_right: i32,
    code_lifetime_secs: i64,
    fully_reset_persistent_state: bool,
) {
    let overrides = overrides
        .as_object_mut()
        .expect("Can insert only into object");
    overrides.insert("start_pairing_overrides".to_owned(), json!({}));
    let overrides = overrides["start_pairing_overrides"]
        .as_object_mut()
        .unwrap();
    overrides.insert("family".to_owned(), json!(family_name));
    overrides.insert("codes_range_left".to_owned(), json!(codes_range_left));
    overrides.insert("codes_range_right".to_owned(), json!(codes_range_right));
    overrides.insert("lifetime".to_owned(), json!(code_lifetime_secs));
    overrides.insert(
        "reset_persistent_state".to_owned(),
        json!(fully_reset_persistent_state),
    );
}

pub struct StartPairingCmdHandler {
    pairing_codes_creator: Arc<Mutex<DefaultPairingCodeCreatorImpl>>,
}

impl StartPairingCmdHandler {
    pub fn new(
        overrides: &JsonValue,
        connection: &dyn DBConnection,
    ) -> Result<StartPairingCmdHandler, Error> {
        let (left, right, family, lifetime, reset) =
            match &overrides["start_pairing_overrides"].as_object() {
                Some(overrides) => (
                    overrides["codes_range_left"].as_i64().unwrap() as i32,
                    overrides["codes_range_right"].as_i64().unwrap() as i32,
                    overrides["family"].as_str().unwrap(),
                    overrides["lifetime"].as_i64().unwrap(),
                    overrides["reset_persistent_state"].as_bool().unwrap(),
                ),
                None => (
                    0,
                    9999,
                    PAIRING_CODES_FAMILY_NAME,
                    PAIRING_CODES_LIFETIME_SECS,
                    false,
                ),
            };

        let pairing_codes_creator =
            pairing_code_creator::new(family.to_owned(), left, right, lifetime)?;
        if reset {
            pairing_codes_creator.fully_reset_persistent_state(connection)?;
        }
        let pairing_codes_creator = Arc::new(Mutex::new(pairing_codes_creator));
        Ok(StartPairingCmdHandler {
            pairing_codes_creator,
        })
    }
}

impl CmdHandler for StartPairingCmdHandler {
    fn handle(
        &self,
        args: HashMap<String, String>,
        connection: BorrowedDBConnection,
        _config: Config,
        _http_client: Arc<HttpClient>,
    ) -> Box<dyn Future<Item = JsonValue, Error = RequestError> + Send> {
        let user_future = done(extract_user_from_query_args(&args, &connection));
        let pairing_codes_creator = self.pairing_codes_creator.clone();
        let result = user_future.and_then(|user| {
                let now_source = DefaultNowSource{};
                done(now_source.now_secs())
                    .map_err(|err|err.into())
                    .join(ok(user))
            })
            .and_then(move |(now, user)| {
                // Note: we use PAIRING_CODES_LIFETIME_USER_VISIBLE_SECS here even though
                // the real lifetime is PAIRING_CODES_LIFETIME_SECS. Reasoning - there's
                // network latency and we don't want the client app to think that pairing
                // is still possible when it's not
                // (PAIRING_CODES_LIFETIME_USER_VISIBLE_SECS < PAIRING_CODES_LIFETIME_SECS).
                let pairing_code_expiration_date = now + PAIRING_CODES_LIFETIME_USER_VISIBLE_SECS;
                let pairing_codes_creator = pairing_codes_creator.lock().expect("Expecting ok mutex");
                done(pairing_codes_creator.borrow_pairing_code(&user, &connection))
                    .map_err(|err| err.into())
                    .join(ok(pairing_code_expiration_date))
                    .map(|(pairing_code, pairing_code_expiration_date)| {
                        json!({
                            constants::FIELD_NAME_STATUS: constants::FIELD_STATUS_OK,
                            constants::FIELD_NAME_PAIRING_CODE: pairing_code,
                            constants::FIELD_NAME_PAIRING_CODE_EXPIRATION_DATE: pairing_code_expiration_date,
                        })
                    })
                },
            );
        Box::new(result)
    }
}

#[cfg(test)]
#[path = "./start_pairing_cmd_handler_test.rs"]
mod start_pairing_cmd_handler_test;
