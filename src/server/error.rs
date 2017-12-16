use uuid;
use db;

error_chain! {
    links {
        Another(db::error::Error, db::error::ErrorKind);
    }

    errors {
        // TODO: ensure that panic with these errors shows parent errors correctly (with stacks)
        DeviceIdDuplicationError(device_id: uuid::Uuid, db_error: db::error::Error) {
            description("Device ID duplication"),
            display("Device ID duplication, ID: {}, parent err: {:?}", device_id, db_error),
        }

        AppUserUniqueIdCreationError(db_error: db::error::Error) {
            description("Couldn't create unique ID for AppUser"),
            display("Couldn't create unique ID for AppUser, parent err: {:?}", db_error),
        }
    }
}