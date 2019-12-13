use diesel;
use diesel::Connection;
use diesel::pg::PgConnection;

use super::error::Error;
use super::error::ErrorKind;
use config::Config;

/// Trait used as a DB connection, is used to perform any actions with the DB.
/// Trait has a single method which would be private if Rust allowed it. Instead, the method itself
/// returns an object which has a private method.
/// That private method is what actually called by DB-related functions to perform operations on the DB.
pub trait DBConnection {
    fn underlying_connection_source(&self) -> &UnderlyingConnectionSource;
}

pub struct UnderlyingConnectionSource {
    diesel_connection: PgConnection,
}
impl UnderlyingConnectionSource {
    pub (in super) fn diesel_connection(&self) -> &PgConnection {
        return &self.diesel_connection;
    }
}

/// Real implementation of a DB connection. Should be used instead of DBConnection only
/// when you implement some sort of fancy wrapping around DB connections.
pub struct DBConnectionImpl {
    underlying_connection_source: UnderlyingConnectionSource,
}
impl DBConnection for DBConnectionImpl {
    fn underlying_connection_source(&self) -> &UnderlyingConnectionSource {
        return &self.underlying_connection_source;
    }
}

impl DBConnectionImpl {
    pub fn for_client_user(config: &Config) -> Result<DBConnectionImpl, Error> {
        return Self::from_raw_params(config.psql_diesel_url_client_user());
    }

    pub fn for_server_user(config: &Config) -> Result<DBConnectionImpl, Error> {
        return Self::from_raw_params(config.psql_diesel_url_server_user());
    }

    pub fn from_raw_params(raw_params: &str) -> Result<DBConnectionImpl, Error> {
        let diesel_connection = diesel::PgConnection::establish(raw_params);
        match diesel_connection {
            Ok(connection) => {
                let connection_source = UnderlyingConnectionSource {
                    diesel_connection:connection,
                };
                return Ok(DBConnectionImpl {
                    underlying_connection_source: connection_source,
                });
            }
            Err(diesel_error) => {
                return Err(ErrorKind::ConnectionError(diesel_error).into());
            }
        }
    }
}

#[cfg(test)]
#[path = "./connection_test.rs"]
mod connection_test;