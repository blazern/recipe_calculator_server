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
pub mod gp_user;
pub mod migrator;
pub mod transaction;
pub mod util;
pub mod vk_user;

// Implementation details.
use diesel;
fn diesel_connection(connection: &dyn connection::DBConnection) -> &diesel::pg::PgConnection {
    connection
        .underlying_connection_source()
        .diesel_connection()
}

#[cfg(test)]
mod diesel_test;
