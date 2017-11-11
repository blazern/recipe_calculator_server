#[allow(unused_imports)]
use config;

#[warn(dead_code)]
fn get_testing_config() -> config::Config {
    use std;

    let config_path = std::env::var("CONFIG_FILE_PATH");
    let config_path = match config_path {
        Ok(val) => val,
        Err(err) => panic!("Is env var CONFIG_FILE_PATH provided? Error: {:?}", err),
    };

    let mut file = std::fs::OpenOptions::new().read(true).open(config_path).unwrap();
    return config::Config::from(&mut file).unwrap();
}