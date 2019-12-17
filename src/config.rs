extern crate serde_json;

use error::Error;
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    vk_server_token: String,
    psql_url_user_server: String,
    psql_url_user_client: String,
    db_connection_attempts_timeout_seconds: i32,
}

impl Config {
    pub fn new(
        vk_server_token: &str,
        psql_url_user_server: &str,
        psql_url_user_client: &str,
        db_connection_attempts_timeout_seconds: i32,
    ) -> Config {
        Config {
            vk_server_token: vk_server_token.to_string(),
            psql_url_user_server: psql_url_user_server.to_string(),
            psql_url_user_client: psql_url_user_client.to_string(),
            db_connection_attempts_timeout_seconds,
        }
    }

    pub fn from(reader: &mut dyn Read) -> Result<Config, Error> {
        let result: Config = serde_json::from_reader(reader)?;
        return Ok(result);
    }

    pub fn vk_server_token(&self) -> &str {
        return &self.vk_server_token;
    }

    pub fn psql_diesel_url_server_user(&self) -> &str {
        return &self.psql_url_user_server;
    }

    pub fn psql_diesel_url_client_user(&self) -> &str {
        return &self.psql_url_user_client;
    }

    pub fn db_connection_attempts_timeout_seconds(&self) -> i32 {
        return self.db_connection_attempts_timeout_seconds;
    }
}

#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;
