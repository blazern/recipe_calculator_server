use db;

use db::core::transaction;
use error;
use pairing;
use uuid::Uuid;

error_chain! {
    links {
        DBCoreError(db::core::error::Error, db::core::error::ErrorKind);
        DBPoolError(db::pool::error::Error, db::pool::error::ErrorKind);
        BaseError(error::Error, error::ErrorKind);
        PairingError(pairing::error::Error, pairing::error::ErrorKind);
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
        VKTokenCheckFail {
            description("VK token check fail"),
            display("VK token check fail, is given token valid?"),
        }
        VKTokenCheckError(error_code: i64, error_msg: String) {
            description("VK token check error"),
            display("VK token check error, code: {:?}, msg: {:?}", error_code, error_msg),
        }
        GPTokenCheckUnknownError {
            description("GP token check fail"),
            display("GP token check fail, is given token valid?"),
        }
        GPTokenCheckError(error_title: String, error_descr: String) {
            description("GP token check error"),
            display("GP token check error, error tile: {:?}, error description: {:?}", error_title, error_descr),
        }
        VKUidDuplicationError {
            description("VK user already registered"),
            display("VK user already registered"),
        }
        GPUidDuplicationError {
            description("GP user already registered"),
            display("GP user already registered"),
        }
    }
}

impl From<transaction::TransactionError<Error>> for Error {
    fn from(error: transaction::TransactionError<Error>) -> Self {
        match error {
            transaction::TransactionError::DBFail(db_fail) => db_fail.into(),
            transaction::TransactionError::OperationFail(test_error) => test_error,
        }
    }
}
