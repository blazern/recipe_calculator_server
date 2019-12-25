use diesel;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    taken_pairing_code {
        id -> Integer,
        app_user_id -> Integer,
        val -> Integer,
        creation_time -> BigInt,
        family -> VarChar,
    }
}
use self::taken_pairing_code as taken_pairing_code_schema;
use diesel::RunQueryDsl;

#[derive(Insertable)]
#[table_name = "taken_pairing_code"]
pub struct NewTakenPairingCode {
    app_user_id: i32,
    val: i32,
    creation_time: i64,
    family: String,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct TakenPairingCode {
    id: i32,
    app_user_id: i32,
    val: i32,
    creation_time: i64,
    family: String,
}

impl TakenPairingCode {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn app_user_id(&self) -> i32 {
        self.app_user_id
    }

    pub fn val(&self) -> i32 {
        self.val
    }

    pub fn creation_time(&self) -> i64 {
        self.creation_time
    }

    pub fn family(&self) -> &str {
        &self.family
    }
}

pub fn new(app_user: &AppUser, val: i32, creation_time: i64, family: String) -> NewTakenPairingCode {
    let app_user_id = app_user.id();
    NewTakenPairingCode {
        app_user_id, val, creation_time, family
    }
}

pub fn insert(
    code: NewTakenPairingCode,
    connection: &dyn DBConnection,
) -> Result<TakenPairingCode, Error> {
    insert!(
        TakenPairingCode,
        code,
        taken_pairing_code_schema::table,
        diesel_connection(connection)
    )
}

pub fn select_by_id(
    id: i32,
    connection: &dyn DBConnection,
) -> Result<Option<TakenPairingCode>, Error> {
    select_by_column!(
        TakenPairingCode,
        taken_pairing_code_schema::table,
        taken_pairing_code_schema::id,
        id,
        diesel_connection(connection)
    )
}

pub fn select_by_app_user_id(
    app_user_id: i32,
    family: &str,
    connection: &dyn DBConnection
) -> Result<Option<TakenPairingCode>, Error> {
    use db::core::transform_diesel_single_result;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = taken_pairing_code_schema::table
        .filter(taken_pairing_code_schema::app_user_id.eq(app_user_id))
        .filter(taken_pairing_code_schema::family.eq(family))
        .first::<TakenPairingCode>(diesel_connection(connection));
    transform_diesel_single_result(result)
}

pub fn delete_family(family: &str, connection: &dyn DBConnection) -> Result<(), Error> {
    delete_by_column!(
        taken_pairing_code_schema::table,
        taken_pairing_code_schema::family,
        family,
        diesel_connection(connection)
    )
}

pub fn delete_older_than(time: i64, family: &str, connection: &dyn DBConnection) -> Result<Vec<TakenPairingCode>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = diesel::delete(
        taken_pairing_code_schema::table
            .filter(taken_pairing_code_schema::creation_time.lt(time))
            .filter(taken_pairing_code_schema::family.eq(family))
    ).get_results::<TakenPairingCode>(diesel_connection(connection));//.execute(diesel_connection(connection));
    result.map_err(|err| err.into())
}

#[cfg(test)]
#[path = "./taken_pairing_code_test.rs"]
mod taken_pairing_code_test;
