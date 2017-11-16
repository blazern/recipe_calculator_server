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
use db::foodstuff;
use db::foodstuff::Foodstuff;
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

fn create_foodstuff(app_user: &AppUser, app_user_foodstuff_id: i32, name: String)
        -> foodstuff::NewFoodstuff {
    return foodstuff::new(
        &app_user,
        app_user_foodstuff_id,
        name,
        0f32,
        0f32,
        0f32,
        0f32,
        false);
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

#[test]
fn update_macro_works() {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    let uid1 = Uuid::from_str("550a8400-e29b-41d4-a716-446655440003").unwrap();
    let uid2 = Uuid::from_str("550a8400-e29b-41d4-a716-446655440004").unwrap();
    delete_user_by_uid(&uid1);
    delete_user_by_uid(&uid2);
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_none());

    let inserted_user = insert!(AppUser, app_user::new(uid1), schema::app_user::table, &connection).unwrap();
    assert!(select_user_by_uid(&uid1).is_some());
    assert!(select_user_by_uid(&uid2).is_none());

    update_column!(
        AppUser,
        schema::app_user::table,
        schema::app_user::id,
        inserted_user.id(),
        schema::app_user::uid,
        &uid2,
        &connection).unwrap();
    assert!(select_user_by_uid(&uid1).is_none());
    assert!(select_user_by_uid(&uid2).is_some());
}

#[test]
fn update_macro_returns_updated_values() {
    let psql_admin_url = env::var(PSQL_ADMIN_URL).unwrap();
    let connection = PgConnection::establish(&psql_admin_url).unwrap();

    // cleaning up
    let uid = Uuid::from_str("550a8400-e29b-41d4-a716-446655440005").unwrap();
    let user = select_user_by_uid(&uid);
    match user {
        Some(user) => {
            diesel::delete(
                schema::foodstuff::table.filter(
                    schema::foodstuff::app_user_id.eq(user.id())))
                .execute(&connection).unwrap();
        }
        _ => {}
    }
    delete_user_by_uid(&uid);
    assert!(select_user_by_uid(&uid).is_none());

    let user = insert!(AppUser, app_user::new(uid), schema::app_user::table, &connection).unwrap();

    let app_user_foodstuff_id1 = 1;
    let app_user_foodstuff_id2 = 2;
    let app_user_foodstuff_id3 = 3;
    let name1 = "name1";
    let name2 = "name2";
    // Note: second foodstuff uses same name as the first one
    let foodstuff1 =
        create_foodstuff(&user, app_user_foodstuff_id1, name1.to_string());
    let foodstuff2 =
        create_foodstuff(&user, app_user_foodstuff_id2, name1.to_string());
    let foodstuff3 =
        create_foodstuff(&user, app_user_foodstuff_id3, name2.to_string());

    let foodstuff1 = insert!(Foodstuff, foodstuff1, schema::foodstuff::table, &connection).unwrap();
    let foodstuff2 = insert!(Foodstuff, foodstuff2, schema::foodstuff::table, &connection).unwrap();
    let foodstuff3 = insert!(Foodstuff, foodstuff3, schema::foodstuff::table, &connection).unwrap();

    assert_eq!(foodstuff1.name(), foodstuff2.name());
    assert_ne!(foodstuff1.name(), foodstuff3.name());

    let new_name = "new-name";
    let updated_foodstuffs =
        update_column!(
            Foodstuff,
            schema::foodstuff::table,
            schema::foodstuff::name,
            &name1,
            schema::foodstuff::name,
            &new_name,
            &connection).unwrap();

    assert_eq!(2, updated_foodstuffs.len());
    assert!(updated_foodstuffs.iter().all(|foodstuff| foodstuff.name() == new_name));
    assert!(updated_foodstuffs.iter().any(|foodstuff| foodstuff.id() == foodstuff1.id()));
    assert!(updated_foodstuffs.iter().any(|foodstuff| foodstuff.id() == foodstuff2.id()));
}