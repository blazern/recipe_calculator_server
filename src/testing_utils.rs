extern crate serde_json;
use std::sync::Once;
use futures::Future;
use config;

#[cfg(test)]
pub fn testing_config() -> config::Config {
    use std;

    let config_path = std::env::var("CONFIG_FILE_PATH");
    let config_path = match config_path {
        Ok(val) => val,
        Err(err) => panic!("Is env var CONFIG_FILE_PATH provided? Error: {:?}", err),
    };

    let mut file = std::fs::OpenOptions::new().read(true).open(config_path).unwrap();
    let result = config::Config::from(&mut file).unwrap();

    let config_json = serde_json::to_string_pretty(&result).unwrap();
    static START: Once = Once::new();
    START.call_once(|| {
        println!("Received testing config:\n{}", config_json);
    });

    return result;
}

#[cfg(test)]
pub fn exhaust_future<Fut, Item, Error>(future: Fut)
        -> Result<Item, Error>
        where Fut: Future<Item=Item, Error=Error>,
              Error: std::fmt::Debug {
    use tokio_core;
    let mut tokio_core = tokio_core::reactor::Core::new().unwrap();
    return tokio_core.run(future);
}