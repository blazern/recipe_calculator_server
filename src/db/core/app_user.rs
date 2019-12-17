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
        client_token -> Uuid,
    }
}
use self::app_user as app_user_schema;

#[derive(Insertable)]
#[table_name = "app_user"]
pub struct NewAppUser {
    uid: Uuid,
    name: String,
    client_token: Uuid,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct AppUser {
    id: i32,
    uid: Uuid,
    name: String,
    client_token: Uuid,
}

impl AppUser {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn uid(&self) -> &Uuid {
        &self.uid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn client_token(&self) -> &Uuid {
        &self.client_token
    }
}

pub fn new(uid: Uuid, name: String, client_token: Uuid) -> NewAppUser {
    NewAppUser {
        uid,
        name,
        client_token,
    }
}

pub fn insert(app_user: NewAppUser, connection: &dyn DBConnection) -> Result<AppUser, Error> {
    return insert!(
        AppUser,
        app_user,
        app_user_schema::table,
        diesel_connection(connection)
    );
}

pub fn select_by_id(id: i32, connection: &dyn DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::id,
        id,
        diesel_connection(connection)
    );
}

pub fn select_by_uid(uid: &Uuid, connection: &dyn DBConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(
        AppUser,
        app_user_schema::table,
        app_user_schema::uid,
        uid,
        diesel_connection(connection)
    );
}

#[cfg(test)]
#[path = "./app_user_test.rs"]
mod app_user_test;
