use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::device;
use crate::db::core::fcm_token;
use crate::db::core::foodstuff;
use crate::db::core::paired_partners;
use crate::db::core::paired_partners::PairingState;
use crate::db::core::testing_util as dbtesting_utils;
use crate::db::core::util::delete_app_user;
use crate::db::core::vk_user;

#[test]
fn deleting_app_user_deletes_all_related_data() {
    let conn = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let uid1 = Uuid::from_str("00000000-a000-0000-0000-009000000000").unwrap();
    let uid2 = Uuid::from_str("00000000-a000-0000-0000-009000000001").unwrap();
    // Even though the functions is being tested, we still need to somehow
    // clean up data before testing.
    delete_app_user(&uid1, &conn).unwrap();
    delete_app_user(&uid2, &conn).unwrap();

    let app_user1 = app_user::insert(
        app_user::new(uid1.clone(), "name".to_string(), Uuid::new_v4()),
        &conn,
    )
    .unwrap();
    let app_user2 = app_user::insert(
        app_user::new(uid2.clone(), "name".to_string(), Uuid::new_v4()),
        &conn,
    )
    .unwrap();
    let device = device::insert(device::new(Uuid::new_v4(), &app_user1), &conn).unwrap();
    let vk_user = vk_user::insert(vk_user::new("vkuid".to_string(), &app_user1), &conn).unwrap();
    let foodstuff1 = foodstuff::insert(
        foodstuff::new(&app_user1, 1, "name".to_string(), 1, 2, 3, 4, true),
        &conn,
    )
    .unwrap();
    let foodstuff2 = foodstuff::insert(
        foodstuff::new(&app_user1, 2, "name2".to_string(), 1, 2, 3, 4, true),
        &conn,
    )
    .unwrap();
    let fcm_token = fcm_token::insert(fcm_token::new("val".to_owned(), &app_user1), &conn).unwrap();

    let paired_partners1 = paired_partners::new(&app_user1, &app_user2, PairingState::Done, 123);
    let paired_partners2 =
        paired_partners::new(&app_user2, &app_user1, PairingState::NotConfirmed, 321);
    paired_partners::insert(paired_partners1, &conn).unwrap();
    paired_partners::insert(paired_partners2, &conn).unwrap();

    assert!(app_user::select_by_uid(&uid1, &conn).unwrap().is_some());
    assert!(device::select_by_id(device.id(), &conn).unwrap().is_some());
    assert!(vk_user::select_by_id(vk_user.id(), &conn)
        .unwrap()
        .is_some());
    assert!(foodstuff::select_by_id(foodstuff1.id(), &conn)
        .unwrap()
        .is_some());
    assert!(foodstuff::select_by_id(foodstuff2.id(), &conn)
        .unwrap()
        .is_some());
    assert!(fcm_token::select_by_id(fcm_token.id(), &conn)
        .unwrap()
        .is_some());
    assert!(
        paired_partners::select_by_partners_user_ids(app_user1.id(), app_user2.id(), &conn)
            .unwrap()
            .is_some()
    );
    assert!(
        paired_partners::select_by_partners_user_ids(app_user2.id(), app_user1.id(), &conn)
            .unwrap()
            .is_some()
    );
    delete_app_user(&uid1, &conn).unwrap();
    assert!(app_user::select_by_uid(&uid1, &conn).unwrap().is_none());
    assert!(device::select_by_id(device.id(), &conn).unwrap().is_none());
    assert!(vk_user::select_by_id(vk_user.id(), &conn)
        .unwrap()
        .is_none());
    assert!(foodstuff::select_by_id(foodstuff1.id(), &conn)
        .unwrap()
        .is_none());
    assert!(foodstuff::select_by_id(foodstuff2.id(), &conn)
        .unwrap()
        .is_none());
    assert!(fcm_token::select_by_id(fcm_token.id(), &conn)
        .unwrap()
        .is_none());
    assert!(
        paired_partners::select_by_partners_user_ids(app_user1.id(), app_user2.id(), &conn)
            .unwrap()
            .is_none()
    );
    assert!(
        paired_partners::select_by_partners_user_ids(app_user2.id(), app_user1.id(), &conn)
            .unwrap()
            .is_none()
    );
}
