use diesel;
use uuid::Uuid;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    device {
        id -> Integer,
        uuid -> Uuid,
        app_user_id -> Integer,
    }
}
use self::device as device_schema;

#[derive(Insertable)]
#[table_name = "device"]
pub struct NewDevice {
    uuid: Uuid,
    app_user_id: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct Device {
    id: i32,
    uuid: Uuid,
    app_user_id: i32,
}

impl Device {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn uuid(&self) -> &Uuid {
        return &self.uuid;
    }

    pub fn app_user_id(&self) -> i32 {
        return self.app_user_id;
    }
}

pub fn new(uuid: Uuid, app_user: &AppUser) -> NewDevice {
    NewDevice {
        uuid: uuid,
        app_user_id: app_user.id(),
    }
}

pub fn insert(device: NewDevice, connection: &dyn DBConnection) -> Result<Device, Error> {
    return insert!(
        Device,
        device,
        device_schema::table,
        diesel_connection(connection)
    );
}

pub fn select_by_id(id: i32, connection: &dyn DBConnection) -> Result<Option<Device>, Error> {
    return select_by_column!(
        Device,
        device_schema::table,
        device_schema::id,
        id,
        diesel_connection(connection)
    );
}

pub fn select_by_uuid(uuid: &Uuid, connection: &dyn DBConnection) -> Result<Option<Device>, Error> {
    return select_by_column!(
        Device,
        device_schema::table,
        device_schema::uuid,
        uuid,
        diesel_connection(connection)
    );
}

pub fn delete_by_id(id: i32, connection: &dyn DBConnection) -> Result<(), Error> {
    return delete_by_column!(
        device_schema::table,
        device_schema::id,
        id,
        diesel_connection(connection)
    );
}

#[cfg(test)]
#[path = "./device_test.rs"]
mod device_test;
