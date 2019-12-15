use diesel;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    vk_user {
        id -> Integer,
        vk_uid -> VarChar,
        app_user_id -> Integer,
    }
}
use self::vk_user as vk_user_schema;

#[derive(Insertable)]
#[table_name="vk_user"]
pub struct NewVkUser {
    vk_uid: String,
    app_user_id: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct VkUser {
    id: i32,
    vk_uid: String,
    app_user_id: i32,
}

impl VkUser {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn vk_uid(&self) -> &str {
        return &self.vk_uid;
    }

    pub fn app_user_id(&self) -> i32 {
        return self.app_user_id;
    }
}

pub fn new(vk_uid: String, app_user: &AppUser) -> NewVkUser {
    NewVkUser {
        vk_uid,
        app_user_id: app_user.id(),
    }
}

pub fn insert(vk_user: NewVkUser, connection: &DBConnection) -> Result<VkUser, Error> {
    return insert!(VkUser, vk_user, vk_user_schema::table, diesel_connection(connection));
}

pub fn select_by_id(id: i32, connection: &DBConnection) -> Result<Option<VkUser>, Error> {
    return select_by_column!(VkUser, vk_user_schema::table, vk_user_schema::id, id, diesel_connection(connection));
}

pub fn select_by_vk_uid(uid: &str, connection: &DBConnection) -> Result<Option<VkUser>, Error> {
    return select_by_column!(VkUser, vk_user_schema::table, vk_user_schema::vk_uid, uid, diesel_connection(connection));
}

#[cfg(test)]
#[path = "./vk_user_test.rs"]
mod vk_user_test;