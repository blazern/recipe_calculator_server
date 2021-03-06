use crate::error::Error;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    vk_server_token: String,
    fcm_server_token: String,
    psql_url_user_server: String,
    psql_url_user_client: String,
    db_connection_attempts_timeout_seconds: i32,
}

impl Config {
    pub fn new(
        vk_server_token: String,
        fcm_server_token: String,
        psql_url_user_server: String,
        psql_url_user_client: String,
        db_connection_attempts_timeout_seconds: i32,
    ) -> Config {
        Config {
            vk_server_token,
            fcm_server_token,
            psql_url_user_server,
            psql_url_user_client,
            db_connection_attempts_timeout_seconds,
        }
    }

    pub fn from(reader: &mut dyn Read) -> Result<Config, Error> {
        let result: Config = serde_json::from_reader(reader)?;
        Ok(result)
    }

    pub fn vk_server_token(&self) -> &str {
        &self.vk_server_token
    }

    pub fn fcm_server_token(&self) -> &str {
        &self.fcm_server_token
    }

    pub fn psql_diesel_url_server_user(&self) -> &str {
        &self.psql_url_user_server
    }

    pub fn psql_diesel_url_client_user(&self) -> &str {
        &self.psql_url_user_client
    }

    pub fn db_connection_attempts_timeout_seconds(&self) -> i32 {
        self.db_connection_attempts_timeout_seconds
    }
}

#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;
