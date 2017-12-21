use diesel;
use uuid::Uuid;

use schema;
use schema::app_user;

use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

#[derive(Insertable)]
#[table_name="app_user"]
pub struct NewAppUser {
    uid: Uuid,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct AppUser {
    id: i32,
    uid: Uuid,
}

impl AppUser {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn uid(&self) -> &Uuid {
        return &self.uid;
    }
}

pub fn new(uid: Uuid) -> NewAppUser {
    NewAppUser{ uid }
}

pub fn insert(app_user: NewAppUser, connection: &DBConnection) -> Result<AppUser, Error> {
    return insert!(AppUser, app_user, schema::app_user::table, diesel_connection(connection));
}

pub fn select_by_id(id: i32, connection: &DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(AppUser, schema::app_user::table, schema::app_user::id, id, diesel_connection(connection));
}

pub fn select_by_uid(uid: &Uuid, connection: &DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(AppUser, schema::app_user::table, schema::app_user::uid, uid, diesel_connection(connection));
}

pub fn delete_by_id(id: i32, connection: &DBConnection) -> Result<(), Error> {
    return delete_by_column!(schema::app_user::table, schema::app_user::id, id, diesel_connection(connection));
}

#[cfg(test)]
#[path = "./app_user_test.rs"]
mod app_user_test;