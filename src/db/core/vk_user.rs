use diesel;

use schema;
use schema::vk_user;
use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

#[derive(Insertable)]
#[table_name="vk_user"]
pub struct NewVkUser {
    vk_uid: i32,
    app_user_id: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct VkUser {
    id: i32,
    vk_uid: i32,
    app_user_id: i32,
}

impl VkUser {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn vk_uid(&self) -> i32 {
        return self.vk_uid;
    }

    pub fn app_user_id(&self) -> i32 {
        return self.app_user_id;
    }
}

pub fn new(vk_uid: i32, app_user: &AppUser) -> NewVkUser {
    NewVkUser{vk_uid: vk_uid, app_user_id: app_user.id() }
}

pub fn insert(vk_user: NewVkUser, connection: &DBConnection) -> Result<VkUser, Error> {
    return insert!(VkUser, vk_user, schema::vk_user::table, diesel_connection(connection));
}

pub fn select_by_id(id: i32, connection: &DBConnection) -> Result<Option<VkUser>, Error> {
    return select_by_column!(VkUser, schema::vk_user::table, schema::vk_user::id, id, diesel_connection(connection));
}

#[cfg(test)]
#[path = "./vk_user_test.rs"]
mod vk_user_test;