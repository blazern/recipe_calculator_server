use db;

use db::core::transaction;
use error;
use uuid::Uuid;

error_chain! {
    links {
        DBCoreError(db::core::error::Error, db::core::error::ErrorKind);
        DBPoolError(db::pool::error::Error, db::pool::error::ErrorKind);
        BaseError(error::Error, error::ErrorKind);
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
        UnsupportedSocialNetwork(social_network_type: String) {
            description("Unsupported social network"),
            display("Unsupported social network: {:?}", social_network_type),
        }
        VKTokenCheckError(error_msg: String, error_code: i64) {
            description("VK token check error"),
            display("VK token check error, code: {:?}, msg: {:?}", error_code, error_msg),
        }
        VkUidDuplicationError {
            description("VK user already registered"),
            display("VK user already registered"),
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