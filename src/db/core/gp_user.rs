use diesel;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    gp_user {
        id -> Integer,
        gp_uid -> VarChar,
        app_user_id -> Integer,
    }
}
use self::gp_user as gp_user_schema;

#[derive(Insertable)]
#[table_name = "gp_user"]
pub struct NewGpUser {
    gp_uid: String,
    app_user_id: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct GpUser {
    id: i32,
    gp_uid: String,
    app_user_id: i32,
}

impl GpUser {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn gp_uid(&self) -> &str {
        &self.gp_uid
    }

    pub fn app_user_id(&self) -> i32 {
        self.app_user_id
    }
}

pub fn new(gp_uid: String, app_user: &AppUser) -> NewGpUser {
    NewGpUser {
        gp_uid,
        app_user_id: app_user.id(),
    }
}

pub fn insert(gp_user: NewGpUser, connection: &dyn DBConnection) -> Result<GpUser, Error> {
    return insert!(
        GpUser,
        gp_user,
        gp_user_schema::table,
        diesel_connection(connection)
    );
}

pub fn select_by_id(id: i32, connection: &dyn DBConnection) -> Result<Option<GpUser>, Error> {
    return select_by_column!(
        GpUser,
        gp_user_schema::table,
        gp_user_schema::id,
        id,
        diesel_connection(connection)
    );
}

pub fn select_by_gp_uid(uid: &str, connection: &dyn DBConnection) -> Result<Option<GpUser>, Error> {
    return select_by_column!(
        GpUser,
        gp_user_schema::table,
        gp_user_schema::gp_uid,
        uid,
        diesel_connection(connection)
    );
}

#[cfg(test)]
#[path = "./gp_user_test.rs"]
mod gp_user_test;
