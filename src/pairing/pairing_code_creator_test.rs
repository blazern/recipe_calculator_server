use std::cell::RefCell;
use std::str::FromStr;
use std::time::SystemTimeError;

use uuid::Uuid;

use crate::db::core::app_user;
use crate::db::core::pairing_code_range;
use crate::db::core::taken_pairing_code;
use crate::db::core::testing_util as dbtesting_utils;
use crate::db::core::util::delete_app_user;

use crate::pairing::error::Error;
use crate::pairing::error::ErrorKind::InvalidBoundsError;
use crate::pairing::error::ErrorKind::OutOfPairingCodes;
use crate::pairing::error::ErrorKind::SameNamedFamilyExistsError;

use crate::utils::now_source::DefaultNowSource;
use crate::utils::now_source::NowSource;

use super::DefaultRandCodeGenerator;
use super::PairingCodeCreator;
use super::RandCodeGenerator;

struct FnNowSource<F>
where
    F: Fn() -> i64,
{
    now_fn: F,
}
impl<F> NowSource for FnNowSource<F>
where
    F: Fn() -> i64,
{
    fn now_secs(&self) -> Result<i64, SystemTimeError> {
        Ok((self.now_fn)())
    }
}

struct FnCodeGenerator<F>
where
    F: Fn(i32, i32) -> i32,
{
    code_fn: F,
}
impl<F> RandCodeGenerator for FnCodeGenerator<F>
where
    F: Fn(i32, i32) -> i32,
{
    fn gen_code(&self, range_left: i32, range_right: i32) -> i32 {
        (self.code_fn)(range_left, range_right)
    }
}

// Cleaning up before tests
fn delete_codes_with_family(family: &str) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    pairing_code_range::delete_family(family, &connection).unwrap();
    taken_pairing_code::delete_family(family, &connection).unwrap();
}

fn delete_user_with_uid(uid: &Uuid) {
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    delete_app_user(&uid, &connection).unwrap();
}

fn create_user_with_uid(uid: &Uuid) -> app_user::AppUser {
    let conn = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let user = app_user::insert(app_user::new(*uid, "".to_owned(), Uuid::new_v4()), &conn);
    user.unwrap()
}

#[test]
fn can_generate_code() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000000").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let creator = super::new(fam.to_owned(), 0, 10, 60 * 4).unwrap();
    let code = creator.borrow_pairing_code(&user, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();
    assert!(0 <= code && code <= 10);
}

#[test]
fn generated_codes_saved_in_db() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000001").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let creator = super::new(fam.to_owned(), 0, 10, 60 * 4).unwrap();

    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn).unwrap();
    assert!(taken_code.is_none());

    let code = creator.borrow_pairing_code(&user, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();

    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn).unwrap();
    assert!(taken_code.is_some());
    let taken_code = taken_code.unwrap();
    assert_eq!(code, taken_code.val());
}

#[test]
fn generated_codes_freed_after_timeout() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000002").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000003").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000004").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let codes_generator = DefaultRandCodeGenerator {};
    let creator = super::new_extended(
        fam.to_owned(),
        0,
        10,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    // Before first generation the code 1 and 2 are not taken
    let taken_code1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    let taken_code2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    assert!(taken_code1.is_none());
    assert!(taken_code2.is_none());

    // After the 2 generations there are taken codes for user1 and user2
    *now.borrow_mut() = 0;
    creator.borrow_pairing_code(&user1, &conn).unwrap();
    // Let a little time pass
    *now.borrow_mut() = codes_life_length / 2;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    let taken_code1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    let taken_code2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    assert!(taken_code1.is_some());
    assert!(taken_code2.is_some());

    // Let a long time pass and generate a third code
    let old_now = *now.borrow();
    *now.borrow_mut() = old_now + codes_life_length - 1;
    creator.borrow_pairing_code(&user3, &conn).unwrap();

    // Third code generation must free first code, because it timeouted ...
    let taken_code1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    assert!(taken_code1.is_none());
    // ... but not the second code, because it's not timeouted yet
    // Third code generation must free first code, because it timeouted ...
    let taken_code2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    assert!(taken_code2.is_some());
}

#[test]
fn out_of_codes_error_returned() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000005").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000006").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000007").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    // [0..1] - there will be 2 codes only
    let creator = super::new(fam.to_owned(), 0, 1, 60 * 4).unwrap();
    creator.borrow_pairing_code(&user1, &conn).unwrap();
    creator.borrow_pairing_code(&user2, &conn).unwrap();
    let code3 = creator.borrow_pairing_code(&user3, &conn);

    match code3 {
        Err(Error(OutOfPairingCodes, _)) => {
            // Ok
        }
        _ => {
            panic!("Out of pairs err expected, got: {:?}", code3);
        }
    }
}

#[test]
fn state_fully_reset_when_time_goes_backwards() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000008").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000009").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);

    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let codes_generator = DefaultRandCodeGenerator {};
    let creator = super::new_extended(
        fam.to_owned(),
        0,
        10,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *now.borrow_mut() = 9999;
    creator.borrow_pairing_code(&user1, &conn).unwrap();
    *now.borrow_mut() = 9998;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    let taken_code1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    let taken_code2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    // Time went backwards - we expect first taken code to be freed because of state reset
    assert!(taken_code1.is_none());
    // But second code should be okay
    assert!(taken_code2.is_some());

    // Verify that free ranges exist only around second code
    let taken_code2 = taken_code2.unwrap().val();
    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    if taken_code2 == 0 {
        assert_eq!(1, free_ranges.len());
        assert_eq!(1, free_ranges[0].left());
        assert_eq!(10, free_ranges[0].right());
    } else if taken_code2 == 10 {
        assert_eq!(1, free_ranges.len());
        assert_eq!(0, free_ranges[0].left());
        assert_eq!(9, free_ranges[0].right());
    } else {
        assert_eq!(2, free_ranges.len());
        assert_eq!(0, free_ranges[0].left());
        assert_eq!(taken_code2 - 1, free_ranges[0].right());
        assert_eq!(taken_code2 + 1, free_ranges[1].left());
        assert_eq!(10, free_ranges[1].right());
    }
}

/// See PairingCodeCreator description to understand what RN1 and RN2 are
#[test]
fn if_rn1_has_wrapping_free_codes_range_then_rn2_is_within_it() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000010").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000011").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000012").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    let wrapping_free_range = pairing_code_range::new(10, 20, fam.to_owned());
    pairing_code_range::insert(wrapping_free_range, &conn).unwrap();

    let left_free_range = pairing_code_range::new(0, 8, fam.to_owned());
    pairing_code_range::insert(left_free_range, &conn).unwrap();
    let taken_code_on_left = taken_pairing_code::new(&user1, 9, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_left, &conn).unwrap();

    let right_free_range = pairing_code_range::new(22, 30, fam.to_owned());
    pairing_code_range::insert(right_free_range, &conn).unwrap();
    let taken_code_on_right = taken_pairing_code::new(&user2, 21, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_right, &conn).unwrap();

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = FnNowSource { now_fn: || 123 };
    let codes_generator = FnCodeGenerator {
        code_fn: |l, r| {
            if l == full_range_left && r == full_range_right {
                // RN1 is requested
                15
            } else {
                // RN2 is requested
                (l + r) / 2
            }
        },
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    let code = creator.borrow_pairing_code(&user3, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();
    assert_eq!((10 + 20) / 2, code);
}

/// See PairingCodeCreator description to understand what RN1 and RN2 are
#[test]
fn if_rn1_has_no_wrapping_free_codes_range_then_rn2_is_within_left_or_right_free_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000013").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000014").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000015").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    // TODO: take all codes of the lacking free ranges after https://trello.com/c/Crc6Cn3B
    // no wrapping free range
    //    let wrapping_free_range = pairing_code_range::new(10, 20, fam.to_owned());
    //    pairing_code_range::insert(wrapping_free_range, &conn).unwrap();

    let left_free_range = pairing_code_range::new(0, 8, fam.to_owned());
    pairing_code_range::insert(left_free_range, &conn).unwrap();
    let taken_code_on_left = taken_pairing_code::new(&user1, 9, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_left, &conn).unwrap();

    let right_free_range = pairing_code_range::new(22, 30, fam.to_owned());
    pairing_code_range::insert(right_free_range, &conn).unwrap();
    let taken_code_on_right = taken_pairing_code::new(&user2, 21, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_right, &conn).unwrap();

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = FnNowSource { now_fn: || 123 };
    let codes_generator = FnCodeGenerator {
        code_fn: |l, r| {
            if l == full_range_left && r == full_range_right {
                // RN1 is requested
                15
            } else {
                // RN2 is requested
                (l + r) / 2
            }
        },
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    let code = creator.borrow_pairing_code(&user3, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();
    assert!(
        (0 + 8) / 2 == code || (22 + 30) / 2 == code,
        "{} was expected to be within a side free range",
        code
    );
}

/// See PairingCodeCreator description to understand what RN1 and RN2 are
#[test]
fn if_rn1_has_only_left_free_range_then_rn2_is_within_left_free_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000016").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000017").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000018").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let _user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    // TODO: take all codes of the lacking free ranges after https://trello.com/c/Crc6Cn3B
    // no wrapping free range
    //    let wrapping_free_range = pairing_code_range::new(10, 20, fam.to_owned());
    //    pairing_code_range::insert(wrapping_free_range, &conn).unwrap();

    let left_free_range = pairing_code_range::new(0, 8, fam.to_owned());
    pairing_code_range::insert(left_free_range, &conn).unwrap();
    let taken_code_on_left = taken_pairing_code::new(&user1, 9, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_left, &conn).unwrap();

    // TODO: take all codes of the lacking free ranges after https://trello.com/c/Crc6Cn3B
    // no right free range
    //    let right_free_range = pairing_code_range::new(22, 30, fam.to_owned());
    //    pairing_code_range::insert(right_free_range, &conn).unwrap();
    //    let taken_code_on_right = taken_pairing_code::new(&user2, 21, 123, fam.to_owned());
    //    taken_pairing_code::insert(taken_code_on_right, &conn).unwrap();

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = FnNowSource { now_fn: || 123 };
    let codes_generator = FnCodeGenerator {
        code_fn: |l, r| {
            if l == full_range_left && r == full_range_right {
                // RN1 is requested
                15
            } else {
                // RN2 is requested
                (l + r) / 2
            }
        },
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    let code = creator.borrow_pairing_code(&user3, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();
    assert_eq!(
        (0 + 8) / 2,
        code,
        "{} was expected to be within the left free range",
        code
    );
}

/// See PairingCodeCreator description to understand what RN1 and RN2 are
#[test]
fn if_rn1_has_only_right_free_range_then_rn2_is_within_right_free_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000019").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000020").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000021").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let _user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    // TODO: take all codes of the lacking free ranges after https://trello.com/c/Crc6Cn3B
    // no wrapping free range
    //    let wrapping_free_range = pairing_code_range::new(10, 20, fam.to_owned());
    //    pairing_code_range::insert(wrapping_free_range, &conn).unwrap();

    // TODO: take all codes of the lacking free ranges after https://trello.com/c/Crc6Cn3B
    // no left free range
    //    let left_free_range = pairing_code_range::new(0, 8, fam.to_owned());
    //    pairing_code_range::insert(left_free_range, &conn).unwrap();
    //    let taken_code_on_left = taken_pairing_code::new(&user1, 9, 123, fam.to_owned());
    //    taken_pairing_code::insert(taken_code_on_left, &conn).unwrap();

    let right_free_range = pairing_code_range::new(22, 30, fam.to_owned());
    pairing_code_range::insert(right_free_range, &conn).unwrap();
    let taken_code_on_right = taken_pairing_code::new(&user2, 21, 123, fam.to_owned());
    taken_pairing_code::insert(taken_code_on_right, &conn).unwrap();

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = FnNowSource { now_fn: || 123 };
    let codes_generator = FnCodeGenerator {
        code_fn: |l, r| {
            if l == full_range_left && r == full_range_right {
                // RN1 is requested
                15
            } else {
                // RN2 is requested
                (l + r) / 2
            }
        },
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    let code = creator.borrow_pairing_code(&user3, &conn).unwrap();
    let code = code.parse::<i32>().unwrap();
    assert_eq!(
        (22 + 30) / 2,
        code,
        "{} was expected to be within the left free range",
        code
    );
}

#[test]
fn new_code_generated_inside_free_range_splits_range_in_two() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000022").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = DefaultNowSource {};
    // It's always 15
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| 15,
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    creator.borrow_pairing_code(&user, &conn).unwrap();

    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn)
        .unwrap()
        .unwrap();

    assert_eq!(15, taken_code.val());

    assert_eq!(2, free_ranges.len());
    assert_eq!(0, free_ranges[0].left());
    assert_eq!(14, free_ranges[0].right());
    assert_eq!(16, free_ranges[1].left());
    assert_eq!(30, free_ranges[1].right());
}

#[test]
fn new_code_generated_on_left_of_free_range_makes_range_smaller_by_1() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000023").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = DefaultNowSource {};
    // It's always full_range_left
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| full_range_left,
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    creator.borrow_pairing_code(&user, &conn).unwrap();

    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn)
        .unwrap()
        .unwrap();

    assert_eq!(full_range_left, taken_code.val());

    assert_eq!(1, free_ranges.len());
    assert_eq!(1, free_ranges[0].left());
    assert_eq!(30, free_ranges[0].right());
}

#[test]
fn new_code_generated_on_right_of_free_range_makes_range_smaller_by_1() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000024").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = DefaultNowSource {};
    // It's always full_range_left
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| full_range_right,
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    creator.borrow_pairing_code(&user, &conn).unwrap();

    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn)
        .unwrap()
        .unwrap();

    assert_eq!(full_range_right, taken_code.val());

    assert_eq!(1, free_ranges.len());
    assert_eq!(0, free_ranges[0].left());
    assert_eq!(29, free_ranges[0].right());
}

#[test]
fn new_code_generated_in_free_range_with_size_of_1_then_it_is_erased() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000025").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let full_range_left = 0;
    let full_range_right = 0;
    let codes_life_length = 10 as i64;
    let now_source = DefaultNowSource {};
    // It's always 0
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| 0,
    };
    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    creator.borrow_pairing_code(&user, &conn).unwrap();

    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    let taken_code = taken_pairing_code::select_by_app_user_id(user.id(), &fam, &conn)
        .unwrap()
        .unwrap();

    assert_eq!(0, taken_code.val());
    assert_eq!(0, free_ranges.len());
}

#[test]
fn state_fully_reset_when_freed_code_has_free_wrapping_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000026").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000027").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000028").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *now.borrow_mut() = 0;
    *next_generated_code.borrow_mut() = 10;
    creator.borrow_pairing_code(&user1, &conn).unwrap();

    *now.borrow_mut() = codes_life_length / 2;
    *next_generated_code.borrow_mut() = 20;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    // Ensure the family is split in 3 parts
    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(3, free_ranges.len(), "ranges: {:?}", free_ranges);

    // Ensure both taken codes are still in DB
    let c1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    let c2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    assert!(c1.is_some());
    assert!(c2.is_some());

    // Replace existing free ranges with a full-wrapping-everything
    // free range
    pairing_code_range::delete_family(&fam, &conn).unwrap();
    let new_range = pairing_code_range::new(full_range_left, full_range_right, fam.to_owned());
    pairing_code_range::insert(new_range, &conn).unwrap();

    // Shift time in such a way that the first code would need to be freed,
    // but the second wouldn't
    *now.borrow_mut() = codes_life_length + 1;
    // Next generated code will be on the right so it wouldn't increase
    // the number of free ranges - we need the third generation only
    // so that the first code would be freed
    *next_generated_code.borrow_mut() = full_range_right;
    // Generate third code
    creator.borrow_pairing_code(&user3, &conn).unwrap();

    // Ensure the state is reset
    let c1 = taken_pairing_code::select_by_app_user_id(user1.id(), &fam, &conn).unwrap();
    let c2 = taken_pairing_code::select_by_app_user_id(user2.id(), &fam, &conn).unwrap();
    assert!(c1.is_none());
    assert!(c2.is_none());
    let free_ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, free_ranges.len());
    assert_eq!(0, free_ranges[0].left());
    assert_eq!(full_range_right - 1, free_ranges[0].right());
}

#[test]
fn when_freed_code_has_free_neighbor_ranges_on_both_sides_they_are_merged() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000029").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000030").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *now.borrow_mut() = 0;
    *next_generated_code.borrow_mut() = 15;
    creator.borrow_pairing_code(&user1, &conn).unwrap();

    // Ensure there're 2 ranges broken apart by our code
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(2, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(14, ranges[0].right());
    assert_eq!(16, ranges[1].left());
    assert_eq!(30, ranges[1].right());

    // Shift time so that the first code would become timeouted
    // and generate a second one
    *now.borrow_mut() = codes_life_length * 10;
    *next_generated_code.borrow_mut() = 30;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    // The previously torn ranges expected to be merged now
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(29, ranges[0].right());
}

#[test]
fn when_freed_code_has_left_free_neighbor_range_then_they_are_merged() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000031").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000032").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *now.borrow_mut() = 0;
    *next_generated_code.borrow_mut() = full_range_right;
    creator.borrow_pairing_code(&user1, &conn).unwrap();

    // Ensure there're 1 range which does't have our taken code
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(29, ranges[0].right());

    // Shift time so that the first code would become timeouted
    // and generate a second one
    *now.borrow_mut() = codes_life_length * 10;
    *next_generated_code.borrow_mut() = full_range_left;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    // The previous range without first code expected to have it now
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(1, ranges[0].left());
    assert_eq!(30, ranges[0].right());
}

#[test]
fn when_freed_code_has_right_free_neighbor_range_then_they_are_merged() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000033").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000034").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *now.borrow_mut() = 0;
    *next_generated_code.borrow_mut() = full_range_left;
    creator.borrow_pairing_code(&user1, &conn).unwrap();

    // Ensure there're 1 range which does't have our taken code
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(1, ranges[0].left());
    assert_eq!(30, ranges[0].right());

    // Shift time so that the first code would become timeouted
    // and generate a second one
    *now.borrow_mut() = codes_life_length * 10;
    *next_generated_code.borrow_mut() = full_range_right;
    creator.borrow_pairing_code(&user2, &conn).unwrap();

    // The previous range without first code expected to have it now
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(29, ranges[0].right());
}

#[test]
fn when_freed_code_has_no_free_neighbor_ranges_then_it_becomes_one() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000035").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000036").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000037").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    // Generate 15, 16 and 14.
    // Make 15 to be timeouted by the third generation.
    *now.borrow_mut() = 0;
    *next_generated_code.borrow_mut() = 15;
    creator.borrow_pairing_code(&user1, &conn).unwrap();
    *now.borrow_mut() = (codes_life_length as f64 * 0.75f64) as i64;
    *next_generated_code.borrow_mut() = 16;
    creator.borrow_pairing_code(&user2, &conn).unwrap();
    *now.borrow_mut() = (codes_life_length as f64 * 1.25f64) as i64;
    *next_generated_code.borrow_mut() = 14;
    creator.borrow_pairing_code(&user3, &conn).unwrap();

    // Verify free ranges states
    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(3, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(13, ranges[0].right());
    assert_eq!(17, ranges[2].left());
    assert_eq!(30, ranges[2].right());
    // !!
    assert_eq!(15, ranges[1].left());
    assert_eq!(15, ranges[1].right());
}

#[test]
fn codes_format() {
    let fam1 = format!("{}{}", file!(), line!());
    let fam2 = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam1);
    delete_codes_with_family(&fam2);
    let uid1 = Uuid::from_str("00000000-0000-0000-0000-002222000038").unwrap();
    let uid2 = Uuid::from_str("00000000-0000-0000-0000-002222000039").unwrap();
    let uid3 = Uuid::from_str("00000000-0000-0000-0000-002222000040").unwrap();
    let uid4 = Uuid::from_str("00000000-0000-0000-0000-002222000041").unwrap();
    delete_user_with_uid(&uid1);
    delete_user_with_uid(&uid2);
    delete_user_with_uid(&uid3);
    delete_user_with_uid(&uid4);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user1 = create_user_with_uid(&uid1);
    let user2 = create_user_with_uid(&uid2);
    let user3 = create_user_with_uid(&uid3);
    let user4 = create_user_with_uid(&uid4);

    let full_range_left = 0;
    let full_range_right = 9999;
    let codes_life_length = 10 as i64;
    let now = RefCell::new(0);
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator1 = super::new_extended(
        fam1.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *next_generated_code.borrow_mut() = 1;
    let code1 = creator1.borrow_pairing_code(&user1, &conn).unwrap();
    *next_generated_code.borrow_mut() = 22;
    let code2 = creator1.borrow_pairing_code(&user2, &conn).unwrap();
    *next_generated_code.borrow_mut() = 333;
    let code3 = creator1.borrow_pairing_code(&user3, &conn).unwrap();
    *next_generated_code.borrow_mut() = 4444;
    let code4 = creator1.borrow_pairing_code(&user4, &conn).unwrap();

    assert_eq!("0001", code1);
    assert_eq!("0022", code2);
    assert_eq!("0333", code3);
    assert_eq!("4444", code4);

    let full_range_right = 999;
    let now_source = FnNowSource {
        now_fn: || *now.borrow(),
    };
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };
    let creator2 = super::new_extended(
        fam2.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *next_generated_code.borrow_mut() = 1;
    let code1 = creator2.borrow_pairing_code(&user1, &conn).unwrap();
    *next_generated_code.borrow_mut() = 22;
    let code2 = creator2.borrow_pairing_code(&user2, &conn).unwrap();
    *next_generated_code.borrow_mut() = 333;
    let code3 = creator2.borrow_pairing_code(&user3, &conn).unwrap();

    assert_eq!("001", code1);
    assert_eq!("022", code2);
    assert_eq!("333", code3);
}

#[test]
fn ranges_table_is_force_reset_when_all_taken_codes_deleted() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);
    let uid = Uuid::from_str("00000000-0000-0000-0000-002222000042").unwrap();
    delete_user_with_uid(&uid);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();
    let user = create_user_with_uid(&uid);

    let full_range_left = 0;
    let full_range_right = 30;
    let codes_life_length = 10 as i64;
    let now_source = FnNowSource { now_fn: || 0 };
    let next_generated_code = RefCell::new(0);
    let codes_generator = FnCodeGenerator {
        code_fn: |_l, _r| *next_generated_code.borrow(),
    };

    let creator = super::new_extended(
        fam.to_owned(),
        full_range_left,
        full_range_right,
        codes_life_length,
        now_source,
        codes_generator,
    )
    .unwrap();

    *next_generated_code.borrow_mut() = 15;
    creator.borrow_pairing_code(&user, &conn).unwrap();

    assert_eq!(
        2,
        pairing_code_range::select_family(&fam, &conn)
            .unwrap()
            .len()
    );
    taken_pairing_code::delete_family(&fam, &conn).unwrap();
    assert_eq!(
        2,
        pairing_code_range::select_family(&fam, &conn)
            .unwrap()
            .len()
    );

    *next_generated_code.borrow_mut() = 30;
    creator.borrow_pairing_code(&user, &conn).unwrap();

    let ranges = pairing_code_range::select_family(&fam, &conn).unwrap();
    assert_eq!(1, ranges.len());
    assert_eq!(0, ranges[0].left());
    assert_eq!(29, ranges[0].right());
}

#[test]
fn multiple_instances_with_same_family_cannot_simultaneously_exist() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);

    {
        // Expecting ok
        let _creator1 = super::new(fam.to_owned(), 0, 1, 2).unwrap();
        // Expecting err
        let creator2_res = super::new(fam.to_owned(), 0, 1, 2);
        match creator2_res {
            Err(Error(SameNamedFamilyExistsError(_), _)) => {
                // Ok
            }
            _ => {
                panic!("Err expected, got: {:?}", creator2_res);
            }
        }
    }
    {
        // Expecting ok
        let _creator3 = super::new(fam.to_owned(), 0, 1, 2).unwrap();
    }
}

#[test]
fn does_not_accept_negative_bounds() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);

    // Expecting err
    let creator2_res = super::new(fam.to_owned(), -1, 1, 2);
    match creator2_res {
        Err(Error(InvalidBoundsError(_), _)) => {
            // Ok
        }
        _ => {
            panic!("Err expected, got: {:?}", creator2_res);
        }
    }
}

#[test]
fn does_not_accept_left_bounds_greater_than_right() {
    let fam = format!("{}{}", file!(), line!());
    delete_codes_with_family(&fam);

    // Expecting err
    let creator2_res = super::new(fam.to_owned(), 10, 9, 2);
    match creator2_res {
        Err(Error(InvalidBoundsError(_), _)) => {
            // Ok
        }
        _ => {
            panic!("Err expected, got: {:?}", creator2_res);
        }
    }
}

#[test]
fn pairing_code_creator_compilation_constraints() {
    let try_build_testcase = trybuild::TestCases::new();
    try_build_testcase.pass("src/pairing/pairing_code_creator_test_send.rs");
    try_build_testcase.compile_fail("src/pairing/pairing_code_creator_test_sync.rs");
}
