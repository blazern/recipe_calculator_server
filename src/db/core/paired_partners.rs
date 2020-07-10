use diesel;

use log::error;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    paired_partners {
        id -> Integer,
        partner1_user_id -> Integer,
        partner2_user_id -> Integer,
        pairing_state -> Integer,
        pairing_start_time -> BigInt,
    }
}
use self::paired_partners as paired_partners_schema;
use crate::db::core::fcm_token::delete_by_user_id;
use diesel::RunQueryDsl;

/// NOTE: the values are stored into DB, so think
/// twice before reusing numeric values.
#[derive(Debug, PartialEq, Clone)]
pub enum PairingState {
    Done = 0,
    NotConfirmed = 1,
}
impl PairingState {
    fn from_number(number: i32) -> Result<Self, ()> {
        match number {
            _ if number == PairingState::Done as i32 => Ok(PairingState::Done),
            _ if number == PairingState::NotConfirmed as i32 => Ok(PairingState::NotConfirmed),
            _ => Err(()),
        }
    }
}

#[derive(Insertable)]
#[table_name = "paired_partners"]
pub struct NewPairedPartners {
    partner1_user_id: i32,
    partner2_user_id: i32,
    pairing_state: i32,
    pairing_start_time: i64,
}

#[derive(Debug, PartialEq, Eq, Queryable)]
pub struct PairedPartners {
    id: i32,
    partner1_user_id: i32,
    partner2_user_id: i32,
    pairing_state: i32,
    pairing_start_time: i64,
}

impl PairedPartners {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn partner1_user_id(&self) -> i32 {
        self.partner1_user_id
    }

    pub fn partner2_user_id(&self) -> i32 {
        self.partner2_user_id
    }

    pub fn pairing_state(&self) -> PairingState {
        PairingState::from_number(self.pairing_state)
            .expect("calls to validate_selection_result exclude invalid numbers")
    }

    pub fn pairing_start_time(&self) -> i64 {
        self.pairing_start_time
    }
}

pub fn new(
    partner1_user: &AppUser,
    partner2_user: &AppUser,
    pairing_state: PairingState,
    pairing_start_time: i64,
) -> NewPairedPartners {
    let partner1_user_id = partner1_user.id();
    let partner2_user_id = partner2_user.id();
    let pairing_state = pairing_state as i32;
    NewPairedPartners {
        partner1_user_id,
        partner2_user_id,
        pairing_state,
        pairing_start_time,
    }
}

#[cfg(test)]
pub fn new_raw_for_tests(
    partner1_user: &AppUser,
    partner2_user: &AppUser,
    pairing_state: i32,
    pairing_start_time: i64,
) -> NewPairedPartners {
    let partner1_user_id = partner1_user.id();
    let partner2_user_id = partner2_user.id();
    NewPairedPartners {
        partner1_user_id,
        partner2_user_id,
        pairing_state,
        pairing_start_time,
    }
}

/// NOTE: even though it doesn't make any sense, it's allowed to have
/// 2 paired_partner rows with relations partner1<->partner2 and partner2<->partner1.
/// Reasoning - it's not easy to prohibit such duplications without locks or sophisticated PSQL
/// features.
pub fn insert(
    code: NewPairedPartners,
    connection: &dyn DBConnection,
) -> Result<PairedPartners, Error> {
    insert!(
        PairedPartners,
        code,
        paired_partners_schema::table,
        diesel_connection(connection)
    )
}

pub fn select_by_id(
    id: i32,
    connection: &dyn DBConnection,
) -> Result<Option<PairedPartners>, Error> {
    let res = select_by_column!(
        PairedPartners,
        paired_partners_schema::table,
        paired_partners_schema::id,
        id,
        diesel_connection(connection)
    );
    validate_selection_result(res, connection)
}

fn validate_selection_result(
    pairing_partners: Result<Option<PairedPartners>, Error>,
    connection: &dyn DBConnection,
) -> Result<Option<PairedPartners>, Error> {
    match pairing_partners {
        Ok(Some(pairing_partners)) => {
            if PairingState::from_number(pairing_partners.pairing_state).is_ok() {
                Ok(Some(pairing_partners))
            } else {
                error!("Data corruption detected in |validate_selection_result|");
                delete_by_user_id(pairing_partners.id, connection)?;
                Ok(None)
            }
        }
        _ => pairing_partners,
    }
}

fn validate_selection_results(
    pairing_partners: Result<Vec<PairedPartners>, Error>,
    connection: &dyn DBConnection,
) -> Result<Vec<PairedPartners>, Error> {
    let mut pairing_partners = match pairing_partners {
        Ok(pairing_partners) => pairing_partners,
        Err(err) => return Err(err),
    };

    let mut first_error: Option<Error> = None;
    pairing_partners.retain(|item| {
        if first_error.is_some() {
            // Doesn't matter now
            return true;
        }

        if PairingState::from_number(item.pairing_state).is_ok() {
            true
        } else {
            error!("Data corruption detected in |validate_selection_results|");
            let del_res = delete_by_user_id(item.id, connection);
            if let Err(err) = del_res {
                first_error = Some(err)
            };
            false
        }
    });

    match first_error {
        Some(err) => Err(err),
        None => Ok(pairing_partners),
    }
}

pub fn select_by_partners_user_ids(
    partner1_user_id: i32,
    partner2_user_id: i32,
    connection: &dyn DBConnection,
) -> Result<Option<PairedPartners>, Error> {
    use crate::db::core::transform_diesel_single_result;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = paired_partners_schema::table
        .filter(paired_partners_schema::partner1_user_id.eq(partner1_user_id))
        .filter(paired_partners_schema::partner2_user_id.eq(partner2_user_id))
        .first::<PairedPartners>(diesel_connection(connection));
    let result = transform_diesel_single_result(result);
    validate_selection_result(result, connection)
}

pub fn select_by_partners_user_ids_and_state(
    partner1_user_id: i32,
    partner2_user_id: i32,
    pairing_state: PairingState,
    connection: &dyn DBConnection,
) -> Result<Option<PairedPartners>, Error> {
    use crate::db::core::transform_diesel_single_result;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = paired_partners_schema::table
        .filter(paired_partners_schema::partner1_user_id.eq(partner1_user_id))
        .filter(paired_partners_schema::partner2_user_id.eq(partner2_user_id))
        .filter(paired_partners_schema::pairing_state.eq(pairing_state as i32))
        .first::<PairedPartners>(diesel_connection(connection));
    let result = transform_diesel_single_result(result);
    validate_selection_result(result, connection)
}

pub fn select_by_partner_user_id_and_state(
    partner_user_id: i32,
    pairing_state: PairingState,
    connection: &dyn DBConnection,
) -> Result<Vec<PairedPartners>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result1 = paired_partners_schema::table
        .filter(paired_partners_schema::partner1_user_id.eq(partner_user_id))
        .filter(paired_partners_schema::pairing_state.eq(pairing_state.clone() as i32))
        .get_results::<PairedPartners>(diesel_connection(connection));

    let result2 = paired_partners_schema::table
        .filter(paired_partners_schema::partner2_user_id.eq(partner_user_id))
        .filter(paired_partners_schema::pairing_state.eq(pairing_state as i32))
        .get_results::<PairedPartners>(diesel_connection(connection));

    let result = match (result1, result2) {
        (Ok(mut result1), Ok(mut result2)) => {
            result1.append(&mut result2);
            Ok(result1)
        }
        (Err(err1), _) => Err(err1),
        (_, Err(err2)) => Err(err2),
    };

    let result = result.map_err(|err| err.into());
    validate_selection_results(result, connection)
}

pub fn delete_by_id(id: i32, connection: &dyn DBConnection) -> Result<(), Error> {
    delete_by_column!(
        paired_partners_schema::table,
        paired_partners_schema::id,
        id,
        diesel_connection(connection)
    )
}

pub fn delete_by_partners_user_ids(
    partner1_user_id: i32,
    partner2_user_id: i32,
    connection: &dyn DBConnection,
) -> Result<(), Error> {
    use crate::db::core::error;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = diesel::delete(
        paired_partners_schema::table
            .filter(paired_partners_schema::partner1_user_id.eq(partner1_user_id))
            .filter(paired_partners_schema::partner2_user_id.eq(partner2_user_id)),
    )
    .execute(diesel_connection(connection));

    let result: Result<(), error::Error> = match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    };
    result
}

pub fn delete_with_state_and_older_than(
    pairing_state: PairingState,
    time: i64,
    connection: &dyn DBConnection,
) -> Result<Vec<PairedPartners>, Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    let result = diesel::delete(
        paired_partners_schema::table
            .filter(paired_partners_schema::pairing_state.eq(pairing_state as i32))
            .filter(paired_partners_schema::pairing_start_time.lt(time)),
    )
    .get_results::<PairedPartners>(diesel_connection(connection));
    result.map_err(|err| err.into())
}

#[cfg(test)]
#[path = "./paired_partners_test.rs"]
mod paired_partners_test;
