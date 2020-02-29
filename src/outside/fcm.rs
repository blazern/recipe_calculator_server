use hyper::Uri;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use super::error::Error;
use super::error::ErrorKind::UnexpectedResponseFormat;
use super::http_client::HttpClient;
use super::http_client::RequestMethod;

// request
// curl -X POST
// -H "Authorization: key=token"
// -H "Content-Type: application/json"
// -d '{ "priority":"high", "data":{ "danil":"FCM Message" }, "to":"client_token"}'
// https://fcm.googleapis.com/fcm/send

// response ok
// {
// "multicast_id":2513734409441993719,
// "success":1,
// "failure":0,
// "canonical_ids":0,
// "results":[{"message_id":"0:1579970411599831%8e9256aef9fd7ecd"}]
// }

// response invalid client token
// {
// "multicast_id":3422611807746461474,
// "success":0,
// "failure":1,
// "canonical_ids":0,
// "results":[{"error":"InvalidRegistration"}]
// }

// response invalid server token
//<HTML>
//<HEAD>
//<TITLE>The request&#39;s Authentication (Server-) Key contained an invalid or malformed FCM-Token (a.k.a. IID-Token).</TITLE>
//</HEAD>
//<BODY BGCOLOR="#FFFFFF" TEXT="#000000">
//<H1>The request&#39;s Authentication (Server-) Key contained an invalid or malformed FCM-Token (a.k.a. IID-Token).</H1>
//<H2>Error 401</H2>
//</BODY>
//</HTML>

pub const FCM_ADDR: &str = "https://fcm.googleapis.com/";

#[derive(Debug, Clone)]
pub enum SendResult {
    Success,
    Error(String),
}

/// Ok(SendResult::Error) for expected errors, Err(..) for unexpected
pub async fn send(
    data: String,
    fcm_token: &str,
    server_fcm_token: &str,
    fcm_address: &str,
    http_client: Arc<HttpClient>,
) -> Result<SendResult, Error> {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_owned(),
        format!("key={}", server_fcm_token),
    );
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());

    let data_json = serde_json::from_str(&data);
    let data = match data_json {
        Ok(data_json) => data_json,
        Err(_) => JsonValue::String(data),
    };

    let body = json!({
        "priority": "high",
        "to": fcm_token,
        "data": data
    });

    let url = Uri::from_str(&format!("{}/fcm/send", fcm_address))?;
    let response = http_client
        .req(url, RequestMethod::Post, headers, Some(body.to_string()))
        .await?;
    send_response_to_send_result(response.body)
}

/// Ok(SendResult::Error) for expected errors, Err(..) for unexpected
pub fn send_response_to_send_result(response: String) -> Result<SendResult, Error> {
    if response.contains("Error 401") {
        return Ok(SendResult::Error(response));
    }
    let response: serde_json::Value = match serde_json::from_str(&response) {
        Ok(response) => response,
        Err(_) => {
            return Err(UnexpectedResponseFormat(format!(
                "Response isn't valid JSON: {}",
                response
            ))
            .into());
        }
    };

    let result = if let JsonValue::Array(results) = &response["results"] {
        if results.len() != 1 {
            return Err(UnexpectedResponseFormat(format!(
                "Unexpected number of results in {}",
                response
            ))
            .into());
        }
        results.first().unwrap()
    } else {
        return Err(UnexpectedResponseFormat(format!(
            "Response doesn't have failure-results: {}",
            response
        ))
        .into());
    };

    if !result["error"].is_null() {
        return Ok(SendResult::Error(response.to_string()));
    }

    Ok(SendResult::Success)
}

#[cfg(test)]
#[path = "./fcm_test.rs"]
mod fcm_test;
