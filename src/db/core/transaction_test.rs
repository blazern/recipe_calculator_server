extern crate diesel;
extern crate uuid;

use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::testing_util as dbtesting_utils;
use db::core::transaction;
use db::core::transaction::TransactionError;

#[derive(Debug)]
struct TestError {
    val: i32,
}

impl TestError {
    fn new() -> TestError {
        TestError{ val: 123 }
    }
    fn with(val: i32) -> TestError {
        TestError{ val }
    }
}

impl From<TransactionError<TestError>> for TestError {
    fn from(error: TransactionError<TestError>) -> Self {
        return match error {
            TransactionError::DBFail(db_fail) => panic!("Unexpected db fail: {:?}", db_fail),
            TransactionError::OperationFail(test_error) => TestError{ val: test_error.val },
        }
    }
}

// Cleaning up before tests
fn delete_entry_with(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let user = app_user::select_by_uid(uid, &connection).unwrap();
    match user {
        Some(user) => {
            app_user::delete_by_id(user.id(), &connection).unwrap();
        }
        _ => {}
    }
}

#[test]
fn transaction_works() {
    let uid = Uuid::from_str("00000000-0000-0000-0000-003000000000").unwrap();
    delete_entry_with(&uid);

    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let transaction_result = transaction::start::<(), _, _>(&connection, || {
        app_user::insert(app_user::new(uid), &connection).unwrap();
        return Err(TestError::new());
    });
    assert!(transaction_result.is_err());

    let user = app_user::select_by_uid(&uid, &connection);
    assert!(user.unwrap().is_none());
}

#[test]
fn returns_correct_error() {
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let val = 100500;
    let transaction_result = transaction::start::<(), TestError, _>(&connection, || {
        return Err(TestError::with(val));
    });

    match transaction_result {
        Err(error) => {
            assert_eq!(val, error.val);
        },
        Ok(_) => {
            panic!("Expecting to fail");
        }
    }
}

#[test]
fn returns_correct_value() {
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let val = 100500;
    let transaction_result = transaction::start::<i32, TestError, _>(&connection, || {
        return Ok(val);
    });

    match transaction_result {
        Ok(result_val) => {
            assert_eq!(val, result_val);
        },
        Err(_) => {
            panic!("Expecting to succeed");
        }
    }
}