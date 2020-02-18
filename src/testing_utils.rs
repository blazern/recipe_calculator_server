use crate::config;
use std::sync::{Mutex, MutexGuard, Once};

lazy_static! {
    static ref TRYBUILD_MUTEX: Mutex<()> = Mutex::new(());
}

#[cfg(test)]
pub fn get_trybuild_lock() -> MutexGuard<'static, ()> {
    TRYBUILD_MUTEX.lock().unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    pub run_trybuild_tests: bool,
}

impl TestingConfig {
    pub fn load() -> TestingConfig {
        let result = Self::load_impl();
        let config_json = serde_json::to_string_pretty(&result).unwrap();
        static START: Once = Once::new();
        START.call_once(|| {
            println!("Used testing config:\n{}", config_json);
        });
        result
    }

    fn load_impl() -> TestingConfig {
        let config_path = std::env::var("TESTING_CONFIG_FILE_PATH");
        let config_path = match config_path {
            Ok(val) => val,
            Err(_err) => return Self::default(),
        };

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&config_path)
            .unwrap();

        let result: TestingConfig = serde_json::from_reader(&file).unwrap();
        result
    }

    fn default() -> TestingConfig {
        TestingConfig {
            run_trybuild_tests: true,
        }
    }
}

#[cfg(test)]
pub fn config_in_tests() -> config::Config {
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
        println!("Received config in tests:\n{}", config_json);
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
