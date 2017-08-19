extern crate serde_json;

use std::io::Read;
use error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    vk_server_token: String,
}

impl Config {
    pub fn new(vk_server_token: &str) -> Config {
        Config{ vk_server_token: vk_server_token.to_string() }
    }

    pub fn from(reader: &mut Read) -> Result<Config, Error> {
        let result: Config = serde_json::from_reader(reader)?;
        return Ok(result);
    }

    pub fn vk_server_token(&self) -> &str {
        return &self.vk_server_token;
    }
}