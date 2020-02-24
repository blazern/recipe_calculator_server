use std::str::FromStr;
use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::paired_partners;
use crate::db::core::paired_partners::PairingState;
use crate::db::core::testing_util as dbtesting_utils;
use crate::db::core::util::delete_app_user;

fn delete_user_with_uid(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    delete_app_user(&uid, &connection).unwrap();
}

#[test]
fn insertion_and_selection() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000000").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000001").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp1 = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp2 = paired_partners::new(&user2, &user1, PairingState::NotConfirmed, 321);
    let pp1 = paired_partners::insert(pp1, &conn).unwrap();
    let pp2 = paired_partners::insert(pp2, &conn).unwrap();

    assert!(pp1.id() > 0);
    assert_eq!(user1.id(), pp1.partner1_user_id());
    assert_eq!(user2.id(), pp1.partner2_user_id());
    assert_eq!(PairingState::Done, pp1.pairing_state());
    assert_eq!(123, pp1.pairing_start_time());

    assert!(pp2.id() > 0);
    assert_eq!(user2.id(), pp2.partner1_user_id());
    assert_eq!(user1.id(), pp2.partner2_user_id());
    assert_eq!(PairingState::NotConfirmed, pp2.pairing_state());
    assert_eq!(321, pp2.pairing_start_time());

    let spp1 = paired_partners::select_by_id(pp1.id(), &conn)
        .unwrap()
        .unwrap();
    let spp2 = paired_partners::select_by_id(pp2.id(), &conn)
        .unwrap()
        .unwrap();

    assert_eq!(pp1, spp1);
    assert_eq!(pp2, spp2);
}

#[test]
fn selection_by_user() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000002").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000003").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp1 = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp2 = paired_partners::new(&user2, &user1, PairingState::NotConfirmed, 321);
    let pp1 = paired_partners::insert(pp1, &conn).unwrap();
    let pp2 = paired_partners::insert(pp2, &conn).unwrap();

    let spp1 = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn)
        .unwrap()
        .unwrap();
    let spp2 = paired_partners::select_by_partners_user_ids(user2.id(), user1.id(), &conn)
        .unwrap()
        .unwrap();

    assert_eq!(pp1, spp1);
    assert_eq!(pp2, spp2);
}

#[test]
fn selection_by_user_and_state() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000004").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000005").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp1 = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp2 = paired_partners::new(&user2, &user1, PairingState::NotConfirmed, 321);
    let pp1 = paired_partners::insert(pp1, &conn).unwrap();
    let pp2 = paired_partners::insert(pp2, &conn).unwrap();

    let spp1_1 = paired_partners::select_by_partners_user_ids_and_state(
        user1.id(),
        user2.id(),
        PairingState::Done,
        &conn,
    )
    .unwrap();
    let spp1_2 = paired_partners::select_by_partners_user_ids_and_state(
        user1.id(),
        user2.id(),
        PairingState::NotConfirmed,
        &conn,
    )
    .unwrap();
    let spp2_1 = paired_partners::select_by_partners_user_ids_and_state(
        user2.id(),
        user1.id(),
        PairingState::Done,
        &conn,
    )
    .unwrap();
    let spp2_2 = paired_partners::select_by_partners_user_ids_and_state(
        user2.id(),
        user1.id(),
        PairingState::NotConfirmed,
        &conn,
    )
    .unwrap();

    assert_eq!(spp1_1.unwrap(), pp1);
    assert!(spp1_2.is_none());
    assert!(spp2_1.is_none());
    assert_eq!(spp2_2.unwrap(), pp2);
}

#[test]
fn row_with_corrupted_state_gets_erased() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000006").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000007").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let invalid_state = 100500;
    let pp = paired_partners::new_raw_for_tests(&user1, &user2, invalid_state, 123);
    paired_partners::insert(pp, &conn).unwrap();

    let spp = paired_partners::select_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(spp.is_none());
}

#[test]
fn deletion_by_partners_ids() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000008").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000009").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp = paired_partners::insert(pp, &conn).unwrap();

    assert!(paired_partners::select_by_id(pp.id(), &conn)
        .unwrap()
        .is_some());
    paired_partners::delete_by_partners_user_ids(user1.id(), user2.id(), &conn).unwrap();
    assert!(paired_partners::select_by_id(pp.id(), &conn)
        .unwrap()
        .is_none());
}

#[test]
fn deletion_of_old_with_certain_state() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000010").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000011").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002210000012").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user3 =
        app_user::insert(app_user::new(uid3, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp1 = paired_partners::new(&user1, &user2, PairingState::Done, 100);
    let pp1 = paired_partners::insert(pp1, &conn).unwrap();
    let pp2 = paired_partners::new(&user1, &user3, PairingState::NotConfirmed, 200);
    let pp2 = paired_partners::insert(pp2, &conn).unwrap();
    let pp3 = paired_partners::new(&user2, &user1, PairingState::Done, 300);
    let pp3 = paired_partners::insert(pp3, &conn).unwrap();
    let pp4 = paired_partners::new(&user2, &user3, PairingState::NotConfirmed, 400);
    let pp4 = paired_partners::insert(pp4, &conn).unwrap();

    let deleted =
        paired_partners::delete_with_state_and_older_than(PairingState::NotConfirmed, 250, &conn)
            .unwrap();
    assert_eq!(1, deleted.len());
    assert_eq!(pp2, deleted[0]);

    assert!(paired_partners::select_by_id(pp1.id(), &conn)
        .unwrap()
        .is_some());
    assert!(paired_partners::select_by_id(pp2.id(), &conn)
        .unwrap()
        .is_none());
    assert!(paired_partners::select_by_id(pp3.id(), &conn)
        .unwrap()
        .is_some());
    assert!(paired_partners::select_by_id(pp4.id(), &conn)
        .unwrap()
        .is_some());
}

#[test]
fn deletion_by_id() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000013").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000014").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp = paired_partners::insert(pp, &conn).unwrap();

    assert!(paired_partners::select_by_id(pp.id(), &conn)
        .unwrap()
        .is_some());
    paired_partners::delete_by_id(pp.id(), &conn).unwrap();
    assert!(paired_partners::select_by_id(pp.id(), &conn)
        .unwrap()
        .is_none());
}

#[test]
fn selection_of_all_paired_partners() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000015").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000016").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002210000017").unwrap();
    let uid4 = Uuid::from_str("00000000-0000-0000-0000-002210000018").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    delete_user_with_uid(&uid4);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user3 =
        app_user::insert(app_user::new(uid3, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user4 =
        app_user::insert(app_user::new(uid4, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let pp1 = paired_partners::new(&user1, &user2, PairingState::Done, 123);
    let pp1 = paired_partners::insert(pp1, &conn).unwrap();
    let pp2 = paired_partners::new(&user3, &user1, PairingState::Done, 123);
    let pp2 = paired_partners::insert(pp2, &conn).unwrap();
    let pp3 = paired_partners::new(&user1, &user4, PairingState::NotConfirmed, 123);
    let pp3 = paired_partners::insert(pp3, &conn).unwrap();

    let selected_pairs =
        paired_partners::select_by_partner_user_id_and_state(user1.id(), PairingState::Done, &conn)
            .unwrap();
    assert_eq!(2, selected_pairs.len());
    assert!(selected_pairs.contains(&pp1));
    assert!(selected_pairs.contains(&pp2));
    assert!(!selected_pairs.contains(&pp3));
}

#[test]
fn selection_of_all_paired_partners_and_data_corruption() {
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002210000019").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002210000020").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 =
        app_user::insert(app_user::new(uid1, "".to_owned(), Uuid::new_v4()), &conn).unwrap();
    let user2 =
        app_user::insert(app_user::new(uid2, "".to_owned(), Uuid::new_v4()), &conn).unwrap();

    let invalid_state = 100500;
    let pp = paired_partners::new_raw_for_tests(&user1, &user2, invalid_state, 123);
    paired_partners::insert(pp, &conn).unwrap();

    let spp =
        paired_partners::select_by_partner_user_id_and_state(user1.id(), PairingState::Done, &conn)
            .unwrap();
    assert!(spp.is_empty());
}
