extern crate diesel;
extern crate uuid;

include!("psql_admin_url.rs.inc");

#[test]
fn transition_rolls_back_progress_when_interrupted() {
    use diesel::Connection;
    use diesel::ExecuteDsl;
    use diesel::ExpressionMethods;
    use diesel::FilterDsl;
    use diesel::pg::PgConnection;
    use std::env;
    use std::str::FromStr;
    use uuid::Uuid;

    use db::app_user;
    use db::app_user::AppUser;
    use error;
    use schema;


    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-646655440001").unwrap();
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    delete_by_column!(
        schema::app_user::table, schema::app_user::uid, uid, &connection).unwrap();

    let invalid_id_value = -1;
    let mut id: i32 = invalid_id_value;
    let transaction_result = connection.transaction::<(), error::Error, _>(|| {
        let new_user = app_user::new(uid);
        let user = insert!(AppUser, new_user, schema::app_user::table, &connection);
        let user = user.unwrap();
        id = user.id();
        Err("Failing transaction by test design")?
    });
    assert!(transaction_result.is_err());
    assert_ne!(invalid_id_value, id);

    let user =
        select_by_column!(AppUser, schema::app_user::table, schema::app_user::id, id, &connection);

    assert!(user.unwrap().is_none());
}

#[test]
fn operations_without_transactions_dont_roll_back() {
    use diesel::Connection;
    use diesel::ExecuteDsl;
    use diesel::ExpressionMethods;
    use diesel::FilterDsl;
    use diesel::pg::PgConnection;
    use std::env;
    use std::str::FromStr;
    use uuid::Uuid;

    use db::app_user;
    use db::app_user::AppUser;
    use schema;


    let uid = Uuid::from_str("550e8400-e29b-41d4-a716-646655440002").unwrap();
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    delete_by_column!(
        schema::app_user::table, schema::app_user::uid, uid, &connection).unwrap();

    let new_user = app_user::new(uid);
    let inserted_user = insert!(AppUser, new_user, schema::app_user::table, &connection);
    assert!(inserted_user.is_ok());

    let selected_user =
        select_by_column!(
            AppUser,
            schema::app_user::table,
            schema::app_user::id,
            inserted_user.unwrap().id(),
            &connection);

    assert!(selected_user.unwrap().is_some());
}