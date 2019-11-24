use super::connection::DBConnection;
use super::error::Error;

embed_migrations!("migrations");

pub fn perform_migrations(connection: &DBConnection) -> Result<(), Error> {
    let result = embedded_migrations::run_with_output(
        connection.diesel_connection(),
        &mut std::io::stdout())?;
    Ok(result)
}