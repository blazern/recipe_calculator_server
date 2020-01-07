use std::time::SystemTimeError;

use db;
use db::core::transaction;
use error;

#[derive(Debug)]
pub enum SystemError {
    Time(SystemTimeError),
}

impl From<transaction::TransactionError<Error>> for Error {
    fn from(error: transaction::TransactionError<Error>) -> Self {
        match error {
            transaction::TransactionError::DBFail(db_fail) => db_fail.into(),
            transaction::TransactionError::OperationFail(test_error) => test_error,
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(error: SystemTimeError) -> Self {
        ErrorKind::UnrecoverableSystemError(SystemError::Time(error)).into()
    }
}

error_chain! {
    links {
        DBCoreError(db::core::error::Error, db::core::error::ErrorKind);
        BaseError(error::Error, error::ErrorKind);
    }

    errors {
        // TODO: ensure that panic with these errors shows parent errors correctly (with stacks)
        OutOfPairingCodes {
            description("Out of pairing codes"),
            display("Out of pairing codes"),
        }
        PersistentStateCorrupted(msg: String) {
            description("Persistent state corrupted"),
            display("Persistent state corrupted: {}", msg),
        }
        UnrecoverableSystemError(err: SystemError) {
            description("Not recoverable system error"),
            display("Not recoverable system error: {:?}", err),
        }
        SameNamedFamilyExistsError(name: String) {
            description("Family with same name already exists"),
            display("Family with same name already exists: {}", name),
        }
        InvalidBoundsError(msg: String) {
            description("Invalid bounds error"),
            display("Invalid bounds error: {}", msg),
        }
    }
}
