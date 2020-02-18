use std::sync::Mutex;

use super::connection::DBConnection;
use super::connection::DBConnectionImpl;
use super::error::Error;
use super::migrator;
use crate::testing_utils::config_in_tests;

// Migrations must not run in parallel (Rust tests are run in parallel) - so we lock them.
lazy_static! {
    static ref MIGRATIONS_MUTEX: Mutex<()> = Mutex::new(());
}

#[cfg(test)]
pub fn testing_connection_for_client_user() -> Result<impl DBConnection, Error> {
    let _migration_lock = MIGRATIONS_MUTEX.lock();
    migrate_with_timeout().unwrap();
    DBConnectionImpl::for_client_user(&config_in_tests())
}

#[cfg(test)]
pub fn testing_connection_for_server_user() -> Result<impl DBConnection, Error> {
    let _migration_lock = MIGRATIONS_MUTEX.lock();
    migrate_with_timeout().unwrap();
    DBConnectionImpl::for_server_user(&config_in_tests())
}

fn migrate_with_timeout() -> Result<(), Error> {
    let config = config_in_tests();
    migrator::migrate_with_timeout(
        config.psql_diesel_url_server_user(),
        config.db_connection_attempts_timeout_seconds() as i64,
    )
}
