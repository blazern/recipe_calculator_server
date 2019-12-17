use db;

error_chain! {
    links {
        DBCoreError(db::core::error::Error, db::core::error::ErrorKind);
    }
}
