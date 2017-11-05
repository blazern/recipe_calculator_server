extern crate serde_json;
extern crate time;

use std::fs::OpenOptions;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use config;

const FILE_PREFIX: &'static str = "/tmp/recipe_calculator_server_test_config";
const VK_SERVER_TOKEN: &'static str = "asdasdasd";
const PSQL_URL: &'static str = "DSASD";

#[test]
fn can_read_config() {
    let filename = FILE_PREFIX.to_string() + &time::precise_time_ns().to_string();
    let mut file = OpenOptions::new().read(true).write(true).create(true).open(filename).unwrap();

    let saved_config = config::Config::new(VK_SERVER_TOKEN, PSQL_URL, PSQL_URL);
    let saved_config_json = serde_json::to_string_pretty(&saved_config).unwrap();

    file.write_all(saved_config_json.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    let read_config = config::Config::from(&mut file).unwrap();
    assert_eq!(saved_config, read_config);
}
