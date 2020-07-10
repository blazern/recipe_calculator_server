use super::connection::DBConnection;
use super::diesel_connection;
use super::error;
use diesel;
use diesel::Connection;

#[derive(Debug)]
pub enum TransactionError<E> {
    OperationFail(E),
    DBFail(error::Error),
}

impl<E> From<diesel::result::Error> for TransactionError<E> {
    fn from(diesel_error: diesel::result::Error) -> Self {
        let db_error = error::ErrorKind::TransactionError(diesel_error);
        TransactionError::DBFail::<E>(db_error.into())
    }
}

pub fn start<T, E, F>(connection: &dyn DBConnection, action: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: From<TransactionError<E>>,
{
    let connection = diesel_connection(connection);
    let transaction_result = connection.transaction::<Result<T, E>, TransactionError<E>, _>(|| {
        let result = action();
        match result {
            Ok(_) => Ok(result),
            Err(error) => Err(TransactionError::OperationFail::<E>(error)),
        }
    });

    match transaction_result {
        Ok(result) => result,
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
#[path = "./transaction_test.rs"]
mod transaction_test;
