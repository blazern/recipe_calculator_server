use futures::Future;
use futures::IntoFuture;
use std;
use std::str::FromStr;
use std::sync::Arc;

use error::Error;
use http_client::HttpClient;
use hyper::Uri;

//curl "https://oauth2.googleapis.com/tokeninfo?id_token=eyJhbGciOi"

//>

// Success:
//{
//"iss": "https://accounts.google.com",
//"azp": "560504820389-e00nqdjqni3rn94cl3r0sfcubavt8pj9.apps.googleusercontent.com",
//"aud": "560504820389-e0pvlp32fn3kn10ud6md0fp533f0170f.apps.googleusercontent.com",
//"sub": "114147567194962866567", <--------------- uid
//"name": "Юлия Жиляева",
//"picture": "https://lh4.googleusercontent.com/-RRXoN7D5Ris/AAAAAAAAAAI/AAAAAAAAAAA/ACHi3rc-UFpiHzd4LzFVJYlzIO9VjQtlIg/s96-c/photo.jpg",
//"given_name": "Юлия",
//"family_name": "Жиляева",
//"locale": "ru",
//"iat": "1576696866",
//"exp": "1576700466",
//"alg": "RS256",
//"kid": "57b1928f2f63329f2e92f4f278f94ee1038c923c",
//"typ": "JWT"
//}

// Error:
//{
//"error": "invalid_token",
//"error_description": "Invalid Value"
//}

#[allow(dead_code)]
pub const ERROR_TITLE_CLIENT_TOKEN_INVALID: &str = "invalid_token";

const URL_TOKENINFO: &str = "https://oauth2.googleapis.com/tokeninfo";
const PARAM_ID_TOKEN: &str = "id_token";

const PARAM_ERROR_TITLE: &str = "error";
const PARAM_ERROR_DESCR: &str = "error_description";
const PARAM_UID: &str = "sub";

#[derive(Debug, Clone)]
pub enum CheckResult {
    Success {
        user_id: String,
    },
    Error {
        error_title: String,
        error_descr: String,
    },
    UnknownError,
}

pub fn check_token_from_server_response<R>(response: R) -> Result<CheckResult, Error>
where
    R: std::io::Read,
{
    let response: serde_json::Value = serde_json::from_reader(response)?;
    match &response[PARAM_ERROR_TITLE] {
        serde_json::Value::String(error_title) => {
            let error_descr = match &response[PARAM_ERROR_DESCR] {
                serde_json::Value::String(error_description) => error_description,
                _ => "",
            };
            let error_title = error_title.to_owned();
            let error_descr = error_descr.to_owned();
            return Ok(CheckResult::Error {
                error_title,
                error_descr,
            });
        }
        _ => {}
    }

    match &response[PARAM_UID] {
        serde_json::Value::String(uid) => Ok(CheckResult::Success {
            user_id: uid.to_string(),
        }),
        _ => Ok(CheckResult::UnknownError), // TODO: return a distinct error type instead of the unknown error
    }
}

pub fn check_token(
    client_token: &str,
    http_client: Arc<HttpClient>,
) -> impl Future<Item = CheckResult, Error = Error> + Send {
    let url = format!("{}?{}={}", URL_TOKENINFO, PARAM_ID_TOKEN, client_token);

    let url = Uri::from_str(&url);
    url.into_future()
        .map_err(|err| err.into())
        .and_then(move |url| http_client.make_request(url))
        .and_then(|response| check_token_from_server_response(response.as_bytes()))
}

#[cfg(test)]
#[path = "./gp_test.rs"]
mod gp_test;
