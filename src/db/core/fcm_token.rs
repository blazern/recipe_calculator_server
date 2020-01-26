use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    fcm_token {
        id -> Integer,
        token_value -> VarChar,
        app_user_id -> Integer,
    }
}
use self::fcm_token as fcm_token_schema;

#[derive(Insertable)]
#[table_name = "fcm_token"]
pub struct NewFcmToken {
    token_value: String,
    app_user_id: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct FcmToken {
    id: i32,
    token_value: String,
    app_user_id: i32,
}

impl FcmToken {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn token_value(&self) -> &str {
        &self.token_value
    }

    pub fn app_user_id(&self) -> i32 {
        self.app_user_id
    }
}

pub fn new(token_value: String, app_user: &AppUser) -> NewFcmToken {
    NewFcmToken {
        token_value,
        app_user_id: app_user.id(),
    }
}

pub fn insert(fcm_token: NewFcmToken, connection: &dyn DBConnection) -> Result<FcmToken, Error> {
    return insert!(
        FcmToken,
        fcm_token,
        fcm_token_schema::table,
        diesel_connection(connection)
    );
}

pub fn select_by_id(id: i32, connection: &dyn DBConnection) -> Result<Option<FcmToken>, Error> {
    return select_by_column!(
        FcmToken,
        fcm_token_schema::table,
        fcm_token_schema::id,
        id,
        diesel_connection(connection)
    );
}

pub fn select_by_user_id(
    user_id: i32,
    connection: &dyn DBConnection,
) -> Result<Option<FcmToken>, Error> {
    return select_by_column!(
        FcmToken,
        fcm_token_schema::table,
        fcm_token_schema::app_user_id,
        user_id,
        diesel_connection(connection)
    );
}

pub fn delete_by_user_id(user_id: i32, connection: &dyn DBConnection) -> Result<(), Error> {
    delete_by_column!(
        fcm_token_schema::table,
        fcm_token_schema::app_user_id,
        user_id,
        diesel_connection(connection)
    )
}

#[cfg(test)]
#[path = "./fcm_token_test.rs"]
mod fcm_token_test;
