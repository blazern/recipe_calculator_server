extern crate serde_json;
extern crate reqwest;
extern crate url;

use error::Error;
use std;

#[allow(dead_code)]
pub const ERROR_CODE_SERVER_TOKEN_INVALID: i64 = 5;
#[allow(dead_code)]
pub const ERROR_CODE_CLIENT_TOKEN_INVALID: i64 = 15;

const HOST_METHOD: &'static str = "https://api.vk.com/method/";
const METHOD_CHECK_TOKEN: &'static str = "secure.checkToken";
const API_VERSION: &'static str = "5.68";

const PARAM_ACCESS_TOKEN: &'static str = "access_token";
const PARAM_TOKEN: &'static str = "token";
const PARAM_API_VERSION: &'static str = "v";

const PARAM_ERROR: &'static str = "error";

#[derive(Debug, Deserialize)]
struct VkError {
    #[serde(rename = "error_code")]
    code: i64,
    #[serde(rename = "error_msg")]
    msg: String,
}

#[derive(Debug, Deserialize)]
struct VkErrorResponse {
    error: VkError,
}

#[derive(Debug, Deserialize)]
struct VkTokenCheckResult {
    success: i64,
    user_id: String,
}

#[derive(Debug)]
pub struct CheckResult {
    is_success: bool,
    user_id: Option<String>,
    error_code: Option<i64>,
    error_msg: Option<String>,
}

impl CheckResult {
    fn from_vk_error(vk_error: VkErrorResponse) -> CheckResult {
        CheckResult {
            is_success: false,
            user_id: None,
            error_code: Some(vk_error.error.code),
            error_msg: Some(vk_error.error.msg),
        }
    }

    fn from_vk_check_result(vk_check_result: VkTokenCheckResult) -> CheckResult {
        let is_success = vk_check_result.success == 1;
        let user_id = if is_success { Some(vk_check_result.user_id) } else { None };
        CheckResult {
            is_success: is_success,
            user_id: user_id,
            error_code: None,
            error_msg: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.is_success
    }

    pub fn user_id(&self) -> &Option<String> {
        &self.user_id
    }

    pub fn error_code(&self) -> &Option<i64> {
        &self.error_code
    }

    pub fn error_msg(&self) -> &Option<String> {
        &self.error_msg
    }
}

pub fn check_token_from_server_response<R>(response: R) -> Result<CheckResult, Error>
        where R: std::io::Read {
    let response: serde_json::Value = serde_json::from_reader(response)?;
    match &response[PARAM_ERROR] {
        &serde_json::Value::Null => {}, // nothing to do, there's no error
        _ => {
            let vk_error: VkErrorResponse = serde_json::from_value(response)?;
            return Ok(CheckResult::from_vk_error(vk_error));
        },
    }

    let vk_check_result: VkTokenCheckResult = serde_json::from_value(response)?;
    return Ok(CheckResult::from_vk_check_result(vk_check_result));
}

pub fn check_token(server_token: &str, client_token: &str) -> Result<CheckResult, Error> {
    let client = reqwest::Client::new()?;
    let url = [HOST_METHOD, METHOD_CHECK_TOKEN].join("");
    let url = url::Url::parse_with_params(&url,
                                          &[(PARAM_ACCESS_TOKEN, server_token),
                                            (PARAM_TOKEN, client_token),
                                            (PARAM_API_VERSION, API_VERSION)])?;

    let response = client.get(url)?.send()?;
    return check_token_from_server_response(response);
}

#[cfg(test)]
#[path = "./vk_test.rs"]
mod vk_test;