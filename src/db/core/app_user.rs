use diesel;
use uuid::Uuid;

use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    app_user {
        id -> Integer,
        uid -> Uuid,
        name -> VarChar,
    }
}
use self::app_user as app_user_schema;

#[derive(Insertable)]
#[table_name="app_user"]
pub struct NewAppUser {
    uid: Uuid,
    name: String
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct AppUser {
    id: i32,
    uid: Uuid,
    name: String
}

impl AppUser {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn uid(&self) -> &Uuid {
        return &self.uid;
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }
}

pub fn new(uid: Uuid, name: &str) -> NewAppUser {
    NewAppUser{ uid, name: name.to_string() }
}

pub fn insert(app_user: NewAppUser, connection: &DBConnection) -> Result<AppUser, Error> {
    return insert!(AppUser, app_user, app_user_schema::table, diesel_connection(connection));
}

pub fn select_by_id(id: i32, connection: &DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(AppUser, app_user_schema::table, app_user_schema::id, id, diesel_connection(connection));
}

pub fn select_by_uid(uid: &Uuid, connection: &DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(AppUser, app_user_schema::table, app_user_schema::uid, uid, diesel_connection(connection));
}

pub fn delete_by_id(id: i32, connection: &DBConnection) -> Result<(), Error> {
    return delete_by_column!(app_user_schema::table, app_user_schema::id, id, diesel_connection(connection));
}

#[cfg(test)]
#[path = "./app_user_test.rs"]
mod app_user_test;