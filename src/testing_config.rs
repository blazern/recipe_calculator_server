extern crate serde_json;
use config;

#[cfg(test)]
pub fn get() -> config::Config {
    use std;

    let config_path = std::env::var("CONFIG_FILE_PATH");
    let config_path = match config_path {
        Ok(val) => val,
        Err(err) => panic!("Is env var CONFIG_FILE_PATH provided? Error: {:?}", err),
    };

    let mut file = std::fs::OpenOptions::new().read(true).open(config_path).unwrap();
    let result = config::Config::from(&mut file).unwrap();
    let config_json = serde_json::to_string_pretty(&result).unwrap();
    println!("Received testing config:\n{}", config_json);
    return result;
}