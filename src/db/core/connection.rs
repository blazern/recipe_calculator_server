use std::env;

use diesel;
use diesel::Connection;
use diesel::pg::PgConnection;

use super::error::Error;
use super::error::ErrorKind;
use config::Config;

pub struct DBConnection {
    diesel_connection: PgConnection,
}

impl DBConnection {
    pub fn for_client_user(config: &Config) -> Result<DBConnection, Error> {
        return Self::from_raw_params(config.psql_diesel_url_client_user());
    }

    pub fn for_server_user(config: &Config) -> Result<DBConnection, Error> {
        return Self::from_raw_params(config.psql_diesel_url_server_user());
    }

    fn from_raw_params(raw_params: &str) -> Result<DBConnection, Error> {
        let diesel_connection = diesel::PgConnection::establish(raw_params);
        match diesel_connection {
            Ok(connection) => {
                return Ok(DBConnection { diesel_connection: connection });
            }
            Err(diesel_error) => {
                return Err(ErrorKind::ConnectionError(diesel_error).into());
            }
        }
    }

    #[cfg(test)]
    pub fn for_admin_user() -> Result<DBConnection, Error> {
        return Self::from_raw_params(&env::var("RECIPE_CALCULATOR_SERVER_PSQL_URL_USER_ADMIN").unwrap());
    }

    pub (in super) fn diesel_connection(&self) -> &PgConnection {
        return &self.diesel_connection;
    }
}

#[cfg(test)]
#[path = "./connection_test.rs"]
mod connection_test;