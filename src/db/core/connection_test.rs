use crate::config::Config;
use crate::db::core::connection::DBConnectionImpl;
use crate::testing_utils::config_in_tests;

#[test]
fn connection_constructs_with_valid_config() {
    let config = config_in_tests();
    let connection = DBConnectionImpl::for_client_user(&config);
    connection.unwrap();
}

#[test]
fn connection_construction_fails_with_invalid_config() {
    let invalid_config = Config::new(
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
        123,
    );
    let connection = DBConnectionImpl::for_client_user(&invalid_config);
    assert!(connection.is_err());
}
