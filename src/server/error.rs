use db;

use uuid::Uuid;
use db::core::transaction;

error_chain! {
    links {
        DBCoreError(db::core::error::Error, db::core::error::ErrorKind);
        DBPoolError(db::pool::error::Error, db::pool::error::ErrorKind);
    }

    errors {
        // TODO: ensure that panic with these errors shows parent errors correctly (with stacks)
        UniqueUuidCreationError(db_error: db::core::error::Error) {
            description("Couldn't create unique UUID"),
            display("Couldn't create unique UUID, parent err: {:?}", db_error),
        }
        DeviceNotFoundError(device_id: Uuid) {
            description("Device ID not found"),
            display("Device ID not found: {:?}", device_id),
        }
    }
}

impl From<transaction::TransactionError<Error>> for Error {
    fn from(error: transaction::TransactionError<Error>) -> Self {
        return match error {
            transaction::TransactionError::DBFail(db_fail) => {
                db_fail.into()
            },
            transaction::TransactionError::OperationFail(test_error) => {
                test_error
            },
        }
    }
}