use crate::config;
use std::sync::Once;

#[cfg(test)]
pub fn testing_config() -> config::Config {
    let config_path = std::env::var("CONFIG_FILE_PATH");
    let config_path = match config_path {
        Ok(val) => val,
        Err(err) => panic!("Is env var CONFIG_FILE_PATH provided? Error: {:?}", err),
    };

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(config_path)
        .unwrap();
    let result = config::Config::from(&mut file).unwrap();

    let config_json = serde_json::to_string_pretty(&result).unwrap();
    static START: Once = Once::new();
    START.call_once(|| {
        println!("Received testing config:\n{}", config_json);
    });

    result
}

#[cfg(test)]
pub fn exhaust_future<Fut, Item, Error>(future: Fut) -> Result<Item, Error>
where
    Fut: futures::future::Future<Output = Result<Item, Error>>,
    Error: std::fmt::Debug,
{
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}
