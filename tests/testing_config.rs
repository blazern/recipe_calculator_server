extern crate recipe_calculator_server;

use std::env;
use std::fs::OpenOptions;
use recipe_calculator_server::config;

const CONFIG_FILE_PATH: &'static str = "CONFIG_FILE_PATH";

pub fn get() -> recipe_calculator_server::config::Config {
    let config_path = env::var(CONFIG_FILE_PATH);
    let config_path = match config_path {
        Ok(val) => val,
        Err(err) => panic!("Is env var {} provided? Error: {:?}", CONFIG_FILE_PATH, err),
    };

    let mut file = OpenOptions::new().read(true).open(config_path).unwrap();
    return config::Config::from(&mut file).unwrap();
}