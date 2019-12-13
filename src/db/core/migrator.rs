use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

use super::connection::DBConnection;
use super::connection::DBConnectionImpl;
use super::error::Error;
use super::error::ErrorKind;

embed_migrations!("migrations");

pub fn perform_migrations(connection: &DBConnection) -> Result<(), Error> {
    let result = embedded_migrations::run_with_output(
        connection.underlying_connection_source().diesel_connection(),
        &mut std::io::stdout())?;
    Ok(result)
}

pub fn migrate_with_timeout(raw_connection_params: &str, timeout_secs: i64) -> Result<(), Error> {
    let connection = get_connection_with_timeout(raw_connection_params, timeout_secs)?;
    perform_migrations(&connection)
}

fn get_connection_with_timeout(raw_connection_params: &str, timeout_secs: i64) -> Result<impl DBConnection, Error> {
    let start_time = now();
    loop {
        let db_connection_result = DBConnectionImpl::from_raw_params(raw_connection_params);
        match db_connection_result {
            Ok(db_connection) => {
                return Ok(db_connection);
            }
            Err(error @ Error(ErrorKind::ConnectionError(_), _)) => {
                let passed_time = now() - start_time;
                if passed_time < timeout_secs {
                    let sleep_length = 5;
                    println!("DB connection failed, going to sleep for {} seconds and retry", sleep_length);
                    let sleep_length = time::Duration::from_secs(sleep_length);
                    thread::sleep(sleep_length);
                } else {
                    return Err(error);
                }
            }
            Err(error) => return Err(error)
        }
    }
}

fn now() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("If there's no system time, something has gone horribly wrong")
        .as_secs()
        as i64;
}
