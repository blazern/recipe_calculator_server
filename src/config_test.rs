use crate::config;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

const FILE_PREFIX: &str = "/tmp/recipe_calculator_server_test_config";
const VK_SERVER_TOKEN: &str = "asdasdasd";
const FCM_SERVER_TOKEN: &str = "dsadsadsa";
const PSQL_URL: &str = "DSASD";
const DB_CONNECTION_TIMEOUT: i32 = 123;

#[test]
fn can_read_config() {
    let filename = FILE_PREFIX.to_string() + &time::precise_time_ns().to_string();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filename)
        .unwrap();

    let saved_config = config::Config::new(
        VK_SERVER_TOKEN.to_owned(),
        FCM_SERVER_TOKEN.to_owned(),
        PSQL_URL.to_owned(),
        PSQL_URL.to_owned(),
        DB_CONNECTION_TIMEOUT,
    );
    let saved_config_json = serde_json::to_string_pretty(&saved_config).unwrap();

    file.write_all(saved_config_json.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    let read_config = config::Config::from(&mut file).unwrap();
    assert_eq!(saved_config, read_config);
}
