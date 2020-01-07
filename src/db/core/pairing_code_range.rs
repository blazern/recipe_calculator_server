use diesel;

use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;
use super::error::ErrorKind::PreconditionsNotSatisfiedError;
use super::transform_diesel_single_result;

table! {
    pairing_code_range {
        id -> Integer,
        #[sql_name = "left_code"]
        left -> Integer,
        #[sql_name = "right_code"]
        right -> Integer,
        family -> VarChar,
    }
}
use self::pairing_code_range as pairing_code_range_schema;

#[derive(Insertable)]
#[table_name = "pairing_code_range"]
pub struct NewPairingCodeRange {
    left: i32,
    right: i32,
    family: String,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct PairingCodeRange {
    id: i32,
    left: i32,
    right: i32,
    family: String,
}

impl PairingCodeRange {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn left(&self) -> i32 {
        self.left
    }

    pub fn right(&self) -> i32 {
        self.right
    }

    pub fn family(&self) -> &str {
        &self.family
    }
}

pub fn new(left: i32, right: i32, family: String) -> NewPairingCodeRange {
    NewPairingCodeRange {
        left,
        right,
        family,
    }
}

pub fn insert(
    range: NewPairingCodeRange,
    connection: &dyn DBConnection,
) -> Result<PairingCodeRange, Error> {
    if range.right < range.left {
        return Err(PreconditionsNotSatisfiedError(format!(
            "Expected left <= right, got {} > {}",
            range.left, range.right
        ))
        .into());
    }
    insert!(
        PairingCodeRange,
        range,
        pairing_code_range_schema::table,
        diesel_connection(connection)
    )
}

pub fn select_by_id(
    id: i32,
    connection: &dyn DBConnection,
) -> Result<Option<PairingCodeRange>, Error> {
    select_by_column!(
        PairingCodeRange,
        pairing_code_range_schema::table,
        pairing_code_range_schema::id,
        id,
        diesel_connection(connection)
    )
}

/// Selects all ranges in family ordered by the |left| value.
pub fn select_family(
    family: &str,
    connection: &dyn DBConnection,
) -> Result<Vec<PairingCodeRange>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    let result = pairing_code_range_schema::table
        .filter(pairing_code_range_schema::family.eq(&family))
        .order(pairing_code_range_schema::right.desc())
        .order(pairing_code_range_schema::left.asc())
        .get_results::<PairingCodeRange>(diesel_connection(connection));

    result.map_err(|err| err.into())
}

pub fn delete_by_id(id: i32, connection: &dyn DBConnection) -> Result<(), Error> {
    delete_by_column!(
        pairing_code_range_schema::table,
        pairing_code_range_schema::id,
        id,
        diesel_connection(connection)
    )
}

pub fn delete_family(family: &str, connection: &dyn DBConnection) -> Result<(), Error> {
    delete_by_column!(
        pairing_code_range_schema::table,
        pairing_code_range_schema::family,
        family,
        diesel_connection(connection)
    )
}

pub fn select_first_to_the_left_of(
    pairing_code: i32,
    family: &str,
    connection: &dyn DBConnection,
) -> Result<Option<PairingCodeRange>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    let result = pairing_code_range_schema::table
        .filter(pairing_code_range_schema::right.lt(pairing_code))
        .filter(pairing_code_range_schema::family.eq(&family))
        .order(pairing_code_range_schema::right.desc())
        .first::<PairingCodeRange>(diesel_connection(connection));
    transform_diesel_single_result(result)
}

pub fn select_first_to_the_right_of(
    pairing_code: i32,
    family: &str,
    connection: &dyn DBConnection,
) -> Result<Option<PairingCodeRange>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    let result = pairing_code_range_schema::table
        .filter(pairing_code_range_schema::left.gt(pairing_code))
        .filter(pairing_code_range_schema::family.eq(&family))
        .order(pairing_code_range_schema::left.asc())
        .first::<PairingCodeRange>(diesel_connection(connection));
    transform_diesel_single_result(result)
}

pub fn select_first_range_with_value_inside(
    pairing_code: i32,
    family: &str,
    connection: &dyn DBConnection,
) -> Result<Option<PairingCodeRange>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;

    let result = pairing_code_range_schema::table
        .filter(pairing_code_range_schema::left.le(pairing_code))
        .filter(pairing_code_range_schema::right.ge(pairing_code))
        .filter(pairing_code_range_schema::family.eq(&family))
        .first::<PairingCodeRange>(diesel_connection(connection));
    transform_diesel_single_result(result)
}

#[cfg(test)]
#[path = "./pairing_code_range_test.rs"]
mod pairing_code_range_test;
