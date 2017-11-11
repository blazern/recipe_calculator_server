use diesel;
use diesel::pg::PgConnection;
use uuid::Uuid;

use schema;
use schema::app_user;
use error::Error;

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

pub fn insert(app_user: NewAppUser, connection: &PgConnection) -> Result<AppUser, Error> {
    return insert!(AppUser, app_user, schema::app_user::table, connection);
}

pub fn select_by_id(id: i32, connection: &PgConnection) -> Result<Option<AppUser>, Error> {
    return select_by_column!(AppUser, schema::app_user::table, schema::app_user::id, id, connection);
}

#[cfg(test)]
#[path = "./app_user_test.rs"]
mod app_user_test;