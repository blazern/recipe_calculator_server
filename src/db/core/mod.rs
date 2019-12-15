#[macro_use]
mod diesel_macro;

#[cfg(test)]
#[macro_use]
pub mod testing_util;

pub mod app_user;
pub mod connection;
pub mod device;
pub mod error;
pub mod foodstuff;
pub mod migrator;
pub mod transaction;
pub mod vk_user;
pub mod util;

// Implementation details.
use diesel;
fn diesel_connection(connection: &connection::DBConnection) -> &diesel::pg::PgConnection {
    return connection.underlying_connection_source().diesel_connection();
}

#[cfg(test)]
mod diesel_test;
