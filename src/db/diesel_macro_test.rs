extern crate diesel;
extern crate uuid;

use std::env;
use std::str::FromStr;
use diesel::Connection;
use diesel::ExecuteDsl;
use diesel::ExpressionMethods;
use diesel::FilterDsl;
use diesel::OptionalExtension;
use diesel::pg::PgConnection;
use uuid::Uuid;

use db::app_user;
use db::app_user::AppUser;
use schema;

include!("psql_admin_url.rs.inc");

fn delete_user_by_uid(uid: &Uuid) {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    diesel::delete(
        schema::app_user::table.filter(
            schema::app_user::uid.eq(uid)))
        .execute(&connection).unwrap();

    assert!(select_user_by_uid(uid).is_none());
}

fn select_user_by_uid(uid: &Uuid) -> Option<AppUser> {
    use diesel::FirstDsl;

    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    return schema::app_user::table.filter(schema::app_user::uid.eq(uid))
        .first::<AppUser>(&connection).optional().unwrap();
}

#[test]
fn insert_macro_works() {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440000").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, schema::app_user::table, &connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());
}

#[test]
fn select_macro_works() {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440001").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, schema::app_user::table, &connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    let result = select_by_column!(AppUser, schema::app_user::table, schema::app_user::uid, &uid, &connection);
    assert!(result.unwrap().is_some());
}

#[test]
fn delete_macro_works() {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440002").unwrap();
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let new_user = app_user::new(uid);
    insert!(AppUser, new_user, schema::app_user::table, &connection).unwrap();
    assert!(select_user_by_uid(&uid).is_some());

    delete_by_column!(schema::app_user::table, schema::app_user::uid, &uid, &connection).unwrap();
    assert!(select_user_by_uid(&uid).is_none());
}