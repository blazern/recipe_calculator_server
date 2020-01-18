use std::collections::HashMap;

use server::constants;
use server::request_error::RequestError;

pub trait HashMapAdditionalOperations {
    fn get_or_request_error(&self, key: &str) -> Result<String, RequestError>;
    fn get_or_empty(&self, key: &str) -> String;
}

#[allow(clippy::implicit_hasher)]
impl HashMapAdditionalOperations for HashMap<std::string::String, std::string::String> {
    fn get_or_request_error(&self, key: &str) -> Result<String, RequestError> {
        let result = self.get(key);
        match result {
            Some(result) => Ok(result.to_string()),
            None => Err(RequestError::new(
                constants::FIELD_STATUS_PARAM_MISSING,
                &format!("No param '{}' in query", key),
            )),
        }
    }
    fn get_or_empty(&self, key: &str) -> String {
        let result = self.get(key);
        match result {
            Some(result) => result.to_string(),
            None => "".to_string(),
        }
    }
}
