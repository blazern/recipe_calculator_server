use std::sync::Arc;

use futures::Future;
use uuid::Uuid;

use error::Error;
use vk;
use http_client::HttpClient;

pub trait UserUuidGenerator : Send {
    fn generate(&self) -> Uuid;
}

pub trait VkTokenChecker : Send {
    fn check_token(&self) -> Box<Future<Item=vk::CheckResult, Error=Error> + Send>;
}

pub fn new_user_uuid_generator_for(overrides: &str) -> Box<UserUuidGenerator> {
    let overridden = maybe_override_uuid_for(&overrides);
    match overridden {
        Some(overridden) => overridden,
        None => Box::new(DefaultUserUuidGenerator{}),
    }
}

pub fn new_vk_token_checker_for(
        overrides: &str,
        client_token: String,
        server_token: String,
        http_client: Arc<HttpClient>) -> Box<VkTokenChecker> {
    let overridden = maybe_override_vk_check_for(&overrides);
    match overridden {
        Some(overridden) => overridden,
        None => Box::new(DefaultVkTokenChecker {
            client_token,
            server_token,
            http_client
        }),
    }
}

//
// Implementation
//

struct DefaultUserUuidGenerator;
impl UserUuidGenerator for DefaultUserUuidGenerator {
    fn generate(&self) -> Uuid {
        return Uuid::new_v4();
    }
}

struct DefaultVkTokenChecker {
    client_token: String,
    server_token: String,
    http_client: Arc<HttpClient>
}
impl VkTokenChecker for DefaultVkTokenChecker {
    fn check_token(&self) -> Box<Future<Item=vk::CheckResult, Error=Error> + Send> {
        Box::new(vk::check_token(&self.server_token, &self.client_token, self.http_client.clone()))
    }
}

//
// Overrides
//

#[cfg(test)]
pub fn create_overrides(uid_override: Option<&Uuid>, vk_token_check_override: Option<&str>) -> String {
    use std::str::FromStr;
    use percent_encoding::percent_encode;
    use percent_encoding::DEFAULT_ENCODE_SET;
    use serde_json;
    use serde_json::Value as JsonValue;

    let mut map = serde_json::map::Map::new();
    match uid_override {
        Some(uid_override) => {
            let mut uid_override_map = serde_json::map::Map::new();
            uid_override_map.insert("uid".to_string(), JsonValue::String(uid_override.to_string()));
            map.insert("uid_override".to_string(), JsonValue::Object(uid_override_map));
        },
        _ => {}
    };

    match vk_token_check_override {
       Some(vk_token_check_override) => {
           let vk_override = JsonValue::from_str(vk_token_check_override).unwrap();
           map.insert("vk_override".to_string(), vk_override);
       },
        _ => {}
    };

    let overrides = JsonValue::Object(map).to_string();
    percent_encode(&overrides.as_bytes(), DEFAULT_ENCODE_SET).to_string()
}

#[cfg(test)]
fn decode_overrides(overrides: &str) -> String {
    use percent_encoding::percent_decode;
    percent_decode(overrides.as_bytes()).decode_utf8().unwrap().to_string()
}

#[cfg(not(test))]
fn maybe_override_uuid_for(_overrides: &str) -> Option<Box<UserUuidGenerator>> { None }
#[cfg(test)]
fn maybe_override_uuid_for(overrides: &str) -> Option<Box<UserUuidGenerator>> {
    use std::str::FromStr;
    use serde_json;
    use serde_json::Value as JsonValue;

    let json = serde_json::from_str(&decode_overrides(overrides));
    let json: JsonValue = match json {
        Ok(json) => json,
        Err(_) => return None,
    };

    match &json["uid_override"] {
        &JsonValue::Object(ref map) => {
            match &map["uid"] {
                &JsonValue::String(ref uid) => {
                    let uid = Uuid::from_str(&uid).unwrap();
                    return Some(Box::new(overriders::UserUuidOverrider{ uid }));
                },
                _ => panic!("Override is found, but it's not a string")
            }
        },
        &JsonValue::Null => {},
        _ => { panic!("Override is found, but it's not an object") }
    };

    None
}

#[cfg(not(test))]
fn maybe_override_vk_check_for(_overrides: &str) -> Option<Box<VkTokenChecker>> { None }
#[cfg(test)]
fn maybe_override_vk_check_for(overrides: &str) -> Option<Box<VkTokenChecker>> {
    use serde_json;
    use serde_json::Value as JsonValue;

    let json = serde_json::from_str(&decode_overrides(overrides));
    let json: JsonValue = match json {
        Ok(json) => json,
        Err(_) => return None,
    };

    match &json["vk_override"] {
        override_json @ &JsonValue::Object(_) => {
            let check_result =
                vk::check_token_from_server_response(
                    override_json.to_string().as_bytes());
            let check_result = check_result.expect(
                &format!("Expected a correct override, got: {}", json.to_string()));
            return Some(Box::new(overriders::VkTokenOverrider{ check_result }));

        },
        &JsonValue::Null => {},
        _ => { panic!("Override is found, but it's not an object") }
    };

    None
}

#[cfg(test)]
mod overriders {
    use uuid::Uuid;
    use futures::Future;
    use futures::future;
    use vk;
    use error::Error;

    pub (in super) struct UserUuidOverrider {
        pub uid: Uuid
    }
    impl super::UserUuidGenerator for UserUuidOverrider {
        fn generate(&self) -> Uuid { self.uid.clone() }
    }

    pub (in super) struct VkTokenOverrider {
        pub check_result: vk::CheckResult,
    }
    impl super::VkTokenChecker for VkTokenOverrider {
        fn check_token(&self) -> Box<Future<Item=vk::CheckResult, Error=Error> + Send> {
            Box::new(future::ok(self.check_result.clone()))
        }
    }
}

