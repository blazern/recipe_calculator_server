use config::Config;
use db::core::connection::DBConnectionImpl;
use testing_utils::testing_config;

#[test]
fn connection_constructs_with_valid_config() {
    let config = testing_config();
    let connection = DBConnectionImpl::for_client_user(&config);
    connection.unwrap();
}

#[test]
fn connection_construction_fails_with_invalid_config() {
    let invalid_config = Config::new("", "", "", 123);
    let connection = DBConnectionImpl::for_client_user(&invalid_config);
    assert!(connection.is_err());
}
