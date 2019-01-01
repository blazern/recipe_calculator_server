// NOTE: target entries are expected to connect to app_user table by a foreign key.
macro_rules! testing_util_delete_entries_with {
    ( $app_user_uid:expr, $entry_table:path, $entry_app_user_id_column:path ) => {
        use db::core::app_user::app_user as app_user_schema;
        
        let connection = DBConnection::for_admin_user().unwrap();
        let raw_connection = diesel_connection(&connection);

        let app_user =
            select_by_column!(
                app_user::AppUser,
                app_user_schema::table,
                app_user_schema::uid,
                $app_user_uid,
                raw_connection);

        let app_user = app_user.unwrap();
        if app_user.is_none() {
            // AppUser already deleted - target entries are connected to it by foreign key, so they are
            // deleted too by now, because otherwise DB wouldn't let us delete
            return;
        }
        let app_user = app_user.unwrap();

        delete_by_column!(
            $entry_table,
            $entry_app_user_id_column,
            app_user.id(),
            raw_connection).unwrap();

        delete_by_column!(
            app_user_schema::table,
            app_user_schema::id,
            app_user.id(),
            raw_connection).unwrap();
    }
}