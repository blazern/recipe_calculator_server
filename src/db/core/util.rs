use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;
use uuid::Uuid;

pub fn delete_app_user(app_user_uid: &Uuid, connection: &dyn DBConnection) -> Result<(), Error> {
    use super::app_user;
    use super::app_user::app_user as app_user_schema;
    use super::device::device as device_schema;
    use super::fcm_token::fcm_token as fcm_token_schema;
    use super::foodstuff::foodstuff as foodstuff_schema;
    use super::gp_user::gp_user as gp_user_schema;
    use super::paired_partners::paired_partners as paired_partners_schema;
    use super::vk_user::vk_user as vk_user_schema;
    let raw_connection = diesel_connection(connection);

    let app_user = app_user::select_by_uid(&app_user_uid, connection)?;
    if app_user.is_none() {
        // AppUser already deleted - target entries are connected to it by foreign key, so they are
        // deleted too by now, because otherwise DB wouldn't let us delete
        return Ok(());
    }
    let app_user = app_user.unwrap();

    delete_by_column!(
        device_schema::table,
        device_schema::app_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        vk_user_schema::table,
        vk_user_schema::app_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        gp_user_schema::table,
        gp_user_schema::app_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        foodstuff_schema::table,
        foodstuff_schema::app_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        fcm_token_schema::table,
        fcm_token_schema::app_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        paired_partners_schema::table,
        paired_partners_schema::partner1_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        paired_partners_schema::table,
        paired_partners_schema::partner2_user_id,
        app_user.id(),
        raw_connection
    )?;

    delete_by_column!(
        app_user_schema::table,
        app_user_schema::id,
        app_user.id(),
        raw_connection
    )?;

    Ok(())
}

#[cfg(test)]
#[path = "./util_test.rs"]
mod util_test;
