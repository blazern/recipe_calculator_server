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
    vk_uid: i32,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct AppUser {
    id: i32,
    uid: Uuid,
    vk_uid: i32,
}

impl AppUser {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn uid(&self) -> &Uuid {
        return &self.uid;
    }

    pub fn vk_uid(&self) -> i32 {
        return self.vk_uid;
    }
}

pub fn new(uid: Uuid, vk_uid: i32) -> NewAppUser {
    NewAppUser{ uid, vk_uid }
}

pub fn insert(app_user: NewAppUser, connection: &PgConnection) -> Result<AppUser, Error> {
    use diesel::LoadDsl;

    let result: AppUser = diesel::insert(&app_user)
        .into(schema::app_user::table)
        .get_result(connection)?;

    return Ok(result);
}

pub fn select_by_id(id: i32, connection: &PgConnection) -> Result<Option<AppUser>, Error> {
    use diesel::FindDsl;
    use diesel::FirstDsl;

    let result = schema::app_user::table.find(id).first::<AppUser>(connection);
    return match result {
        Err(diesel::result::Error::NotFound) => {
            Ok(None)
        }
        Err(error) => {
            Err(error.into())
        }
        Ok(app_user) => {
            Ok(Some(app_user))
        }
    }
}

#[cfg(test)]
#[path = "./app_user_test.rs"]
mod app_user_test;