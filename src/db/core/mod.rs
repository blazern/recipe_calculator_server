#[macro_use]
mod diesel_macro;

#[cfg(test)]
#[macro_use]
pub mod testing_util;

pub mod app_user;
pub mod connection;
pub mod device;
pub mod error;
pub mod fcm_token;
pub mod foodstuff;
pub mod gp_user;
pub mod migrator;
pub mod paired_partners;
pub mod pairing_code_range;
pub mod taken_pairing_code;
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

fn transform_diesel_single_result<T>(
    diesel_result: Result<T, diesel::result::Error>,
) -> Result<Option<T>, error::Error> {
    match diesel_result {
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(error) => Err(error.into()),
        Ok(val) => Ok(Some(val)),
    }
}

#[cfg(test)]
mod diesel_test;
