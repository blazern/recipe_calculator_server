extern crate diesel;
extern crate diesel_migrations;
extern crate uuid;

error_chain! {
    foreign_links {
        // General error for all not specified DB failures.
        DBError(diesel::result::Error);
        DBMigrationError(diesel_migrations::RunMigrationsError);
    }

    errors {
        // TODO: ensure that panic with these errors shows parent errors correctly (with stacks)
        ConnectionError(diesel_error: diesel::ConnectionError) {
            description("Connection couldn't be established"),
            display("Connection couldn't be established: {:?}", diesel_error),
        }
        UniqueViolation(diesel_error: diesel::result::Error) {
            description("Unique constraint violation"),
            display("Unique constraint violation: {:?}", diesel_error),
        }
        TransactionError(diesel_error: diesel::result::Error) {
            description("Transaction error"),
            display("Transaction error: {:?}", diesel_error),
        }
    }
}