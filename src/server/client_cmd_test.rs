extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use http_client::HttpClient;
use server::error::Error;
use server::error::ErrorKind::UniqueUuidCreationError;
use db::core::app_user;
use db::core::testing_util as dbtesting_utils;
use server::client_cmd;
use testing_utils::testing_config;
use testing_utils::exhaust_future;

struct SameUuidGenerator {
    uuid: Uuid,
}

impl SameUuidGenerator {
    fn new(uid: Uuid) -> SameUuidGenerator {
        SameUuidGenerator{ uuid: uid }
    }
}

impl client_cmd::UuidGenerator for SameUuidGenerator {
    fn generate(&mut self) -> Uuid {
        return self.uuid.clone();
    }
}

// Cleaning up before tests
fn delete_app_user_with(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let selected_app_user = app_user::select_by_uid(&uid, &connection);
    match selected_app_user {
        Ok(Some(app_user)) => {
            app_user::delete_by_id(app_user.id(), &connection).unwrap();
        }
        _ => {}
    }
}

#[test]
fn user_registration_works() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000000").unwrap();
    delete_app_user_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let uid_generator = SameUuidGenerator::new(uid.clone());
    let config = testing_config();

    let result =
        client_cmd::register_user_by_uuid_generator(
            "name".to_string(),
            uid_generator,
            config,
            connection,
            Arc::new(HttpClient::new().unwrap()));
    exhaust_future(result).unwrap();

    // making sure app_user is inserted
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = app_user::select_by_uid(&uid, &connection).unwrap().unwrap();
    assert_eq!("name", user.name())
}

#[test]
fn user_registration_returns_duplication_error_on_uid_duplication() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-001000000005").unwrap();
    delete_app_user_with(&uid);

    let result1 =
        client_cmd::register_user_by_uuid_generator(
            "name1".to_string(),
            SameUuidGenerator::new(uid.clone()),
            testing_config(),
            dbtesting_utils::testing_connection_for_client_user().unwrap(),
            Arc::new(HttpClient::new().unwrap()));
    exhaust_future(result1).unwrap();

    let second_registration_result =
        client_cmd::register_user_by_uuid_generator(
            "name2".to_string(),
            SameUuidGenerator::new(uid.clone()),
            testing_config(),
            dbtesting_utils::testing_connection_for_client_user().unwrap(),
            Arc::new(HttpClient::new().unwrap()));
    let second_registration_result = exhaust_future(second_registration_result);

    match second_registration_result {
        Err(Error(UniqueUuidCreationError(_), _)) => {
            // OK
        }
        Err(err) => {
            panic!("Expected error::UniqueUuidCreationError, but got another error: {:?}", err);
        }
        Ok(_) => {
            panic!("Expected error::UniqueUuidCreationError, but registration was successful!");
        }
    }
}

//#[test]
//fn uid_duplication_leaves_vk_user_not_created() {
//    // TODO
//}
//
//#[test]
//fn uid_duplication_leaves_google_user_not_created() {
//  // TODO
//}