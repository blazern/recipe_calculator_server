#[macro_use]
mod diesel_macro;

pub mod app_user;
pub mod connection;
pub mod device;
pub mod error;
pub mod foodstuff;
pub mod vk_user;

// Implementation details.
use diesel;
fn diesel_connection(connection: &connection::DBConnection) -> &diesel::pg::PgConnection {
    return connection.diesel_connection();
}

#[cfg(test)]
mod diesel_test;
