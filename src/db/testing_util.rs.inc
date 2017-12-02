// NOTE: target entries are expected to connect to app_user table by a foreign key.
macro_rules! testing_util_delete_entries_with {
    ( $app_user_uid:expr, $entry_table:path, $entry_app_user_id_column:path ) => {
        let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
        let connection = PgConnection::establish(&psql_admin_url).unwrap();

        let app_user =
            select_by_column!(
                app_user::AppUser,
                schema::app_user::table,
                schema::app_user::uid,
                $app_user_uid,
                &connection);

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
            &connection).unwrap();

        delete_by_column!(
            schema::app_user::table,
            schema::app_user::id,
            app_user.id(),
            &connection).unwrap();
    }
}