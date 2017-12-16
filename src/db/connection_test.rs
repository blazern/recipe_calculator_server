use super::DBConnection;
use config::Config;

include!("../testing_config.rs.inc");

#[test]
fn connection_constructs_with_valid_config() {
    let config = get_testing_config();
    let connection = DBConnection::for_client_user(&config);
    connection.unwrap();
}

#[test]
fn connection_construction_fails_with_invalid_config() {
    let invalid_config = Config::new("", "", "");
    let connection = DBConnection::for_client_user(&invalid_config);
    assert!(connection.is_err());
}