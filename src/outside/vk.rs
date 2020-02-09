use std;
use std::str::FromStr;
use std::sync::Arc;

use hyper::Uri;

use super::error::Error;
use super::http_client::HttpClient;

#[allow(dead_code)]
pub const ERROR_CODE_SERVER_TOKEN_INVALID: i64 = 5;
#[allow(dead_code)]
pub const ERROR_CODE_CLIENT_TOKEN_INVALID: i64 = 15;

const HOST_METHOD: &str = "https://api.vk.com/method/";
const METHOD_CHECK_TOKEN: &str = "secure.checkToken";
const API_VERSION: &str = "5.68";

const PARAM_ACCESS_TOKEN: &str = "access_token";
const PARAM_TOKEN: &str = "token";
const PARAM_API_VERSION: &str = "v";

const PARAM_ERROR: &str = "error";

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

#[derive(Debug, Clone)]
pub enum CheckResult {
    Success { user_id: String },
    Fail,
    Error { error_code: i64, error_msg: String },
}

impl CheckResult {
    fn from_vk_error(vk_error: VkErrorResponse) -> CheckResult {
        CheckResult::Error {
            error_code: vk_error.error.code,
            error_msg: vk_error.error.msg,
        }
    }

    fn from_vk_check_result(vk_check_result: VkTokenCheckResult) -> CheckResult {
        let is_success = vk_check_result.success == 1;
        if is_success {
            CheckResult::Success {
                user_id: vk_check_result.user_id,
            }
        } else {
            CheckResult::Fail
        }
    }
}

pub fn check_token_from_server_response<R>(response: R) -> Result<CheckResult, Error>
where
    R: std::io::Read,
{
    let response: serde_json::Value = serde_json::from_reader(response)?;
    match &response[PARAM_ERROR] {
        serde_json::Value::Null => {} // nothing to do, there's no error
        _ => {
            let vk_error: VkErrorResponse = serde_json::from_value(response)?;
            return Ok(CheckResult::from_vk_error(vk_error));
        }
    }

    let vk_check_result: VkTokenCheckResult = serde_json::from_value(response)?;
    Ok(CheckResult::from_vk_check_result(vk_check_result))
}

pub async fn check_token(
    server_token: String,
    client_token: String,
    http_client: Arc<HttpClient>,
) -> Result<CheckResult, Error> {
    let url = [HOST_METHOD, METHOD_CHECK_TOKEN].join("");
    let url = format!(
        "{}?{}={}&{}={}&{}={}",
        url,
        PARAM_ACCESS_TOKEN,
        server_token,
        PARAM_TOKEN,
        client_token,
        PARAM_API_VERSION,
        API_VERSION
    );

    let url = Uri::from_str(&url)?;
    let response = http_client.req_get(url).await?;
    check_token_from_server_response(response.body.as_bytes())
}

#[cfg(test)]
#[path = "./vk_test.rs"]
mod vk_test;
