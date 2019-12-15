use std::str::FromStr;
use uuid::Uuid;

use db::core::app_user;
use db::core::foodstuff;
use db::core::device;
use db::core::vk_user;
use db::core::testing_util as dbtesting_utils;
use db::core::util::delete_app_user;

#[test]
fn deleting_app_user_deletes_all_related_data() {
    let conn = dbtesting_utils::testing_connection_for_server_user().unwrap();

    let uid = Uuid::from_str("00000000-a000-0000-0000-009000000000").unwrap();
    // Even though the functions is being tested, we still need to somehow
    // clean up data before testing.
    delete_app_user(&uid, &conn).unwrap();

    let app_user = app_user::insert(app_user::new(uid.clone(), "name"), &conn).unwrap();
    let device = device::insert(device::new(Uuid::new_v4(), &app_user), &conn).unwrap();
    let vk_user = vk_user::insert(vk_user::new("vkuid".to_string(), &app_user), &conn).unwrap();
    let foodstuff1 = foodstuff::insert(
        foodstuff::new(&app_user, 1, "name".to_string(), 1, 2, 3, 4, true), &conn).unwrap();
    let foodstuff2 = foodstuff::insert(
        foodstuff::new(&app_user, 2, "name2".to_string(), 1, 2, 3, 4, true), &conn).unwrap();

    assert!(app_user::select_by_uid(&uid, &conn).unwrap().is_some());
    assert!(device::select_by_id(device.id(), &conn).unwrap().is_some());
    assert!(vk_user::select_by_id(vk_user.id(), &conn).unwrap().is_some());
    assert!(foodstuff::select_by_id(foodstuff1.id(), &conn).unwrap().is_some());
    assert!(foodstuff::select_by_id(foodstuff2.id(), &conn).unwrap().is_some());
    delete_app_user(&uid, &conn).unwrap();
    assert!(app_user::select_by_uid(&uid, &conn).unwrap().is_none());
    assert!(device::select_by_id(device.id(), &conn).unwrap().is_none());
    assert!(vk_user::select_by_id(vk_user.id(), &conn).unwrap().is_none());
    assert!(foodstuff::select_by_id(foodstuff1.id(), &conn).unwrap().is_none());
    assert!(foodstuff::select_by_id(foodstuff2.id(), &conn).unwrap().is_none());
}