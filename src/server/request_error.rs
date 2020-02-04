use std::time::SystemTimeError;

use db::core::error::Error as DbCoreError;
use db::core::transaction::TransactionError as DbTransactionError;
use db::pool::error::Error as DbPoolError;
use outside::error::Error as OutsideError;
use pairing::error::Error as PairingError;
use server::error::Error as ServerError;
use server::error::ErrorKind as ServerErrorKind;

use super::constants;

pub struct RequestError {
    status: String,
    error_description: String,
}

impl RequestError {
    pub fn new(status: &str, error_description: &str) -> Self {
        RequestError {
            status: status.to_string(),
            error_description: error_description.to_string(),
        }
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn error_description(&self) -> &str {
        &self.error_description
    }
}

impl From<SystemTimeError> for RequestError {
    fn from(error: SystemTimeError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("System time error: {}", error),
        )
    }
}

impl From<DbCoreError> for RequestError {
    fn from(error: DbCoreError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("DB error: {}", error),
        )
    }
}

impl From<DbPoolError> for RequestError {
    fn from(error: DbPoolError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Pool error: {}", error),
        )
    }
}

impl From<DbTransactionError<RequestError>> for RequestError {
    fn from(error: DbTransactionError<RequestError>) -> Self {
        match error {
            DbTransactionError::DBFail(error) => error.into(),
            DbTransactionError::OperationFail(error) => error,
        }
    }
}

impl From<PairingError> for RequestError {
    fn from(error: PairingError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Internal pairing error: {}", error),
        )
    }
}

impl From<OutsideError> for RequestError {
    fn from(error: OutsideError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INTERNAL_ERROR,
            &format!("Outside service error: {}", error),
        )
    }
}

impl From<ServerError> for RequestError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError(error @ ServerErrorKind::VKUidDuplicationError, _) => RequestError::new(
                constants::FIELD_STATUS_ALREADY_REGISTERED,
                &format!("User already registered: {}", error),
            ),
            ServerError(error @ ServerErrorKind::GPUidDuplicationError, _) => RequestError::new(
                constants::FIELD_STATUS_ALREADY_REGISTERED,
                &format!("User already registered: {}", error),
            ),
            ServerError(error @ ServerErrorKind::VKTokenCheckError(_, _), _) => RequestError::new(
                constants::FIELD_STATUS_TOKEN_CHECK_FAIL,
                &format!("Token check fail: {}", error),
            ),
            ServerError(error @ ServerErrorKind::VKTokenCheckFail {}, _) => RequestError::new(
                constants::FIELD_STATUS_TOKEN_CHECK_FAIL,
                &format!("Token check fail: {}", error),
            ),
            ServerError(error @ ServerErrorKind::GPTokenCheckError(_, _), _) => RequestError::new(
                constants::FIELD_STATUS_TOKEN_CHECK_FAIL,
                &format!("Token check fail: {}", error),
            ),
            ServerError(error @ ServerErrorKind::GPTokenCheckUnknownError {}, _) => {
                RequestError::new(
                    constants::FIELD_STATUS_TOKEN_CHECK_FAIL,
                    &format!("Token check fail: {}", error),
                )
            }
            ServerError(error, _) => RequestError::new(
                constants::FIELD_STATUS_INTERNAL_ERROR,
                &format!("Internal error: {}", error),
            ),
        }
    }
}

impl From<uuid::parser::ParseError> for RequestError {
    fn from(error: uuid::parser::ParseError) -> Self {
        RequestError::new(
            constants::FIELD_STATUS_INVALID_UUID,
            &format!("Invalid UUID: {}", error),
        )
    }
}
