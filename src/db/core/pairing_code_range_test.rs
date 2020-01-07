extern crate diesel;
extern crate uuid;

use db::core::diesel_connection;
use db::core::pairing_code_range;
use db::core::testing_util as dbtesting_utils;

// Cleaning up before tests
fn delete_ranges_with_family(family: &str) {
    use db::core::pairing_code_range::pairing_code_range as pairing_code_range_schema;
    let connection = dbtesting_utils::testing_connection_for_server_user().unwrap();
    let raw_connection = diesel_connection(&connection);
    delete_by_column!(
        pairing_code_range_schema::table,
        pairing_code_range_schema::family,
        family,
        raw_connection
    )
    .unwrap();
}

#[test]
fn insertion_and_selection_work() {
    let test_family = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&test_family);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let left = 0;
    let right = 999;
    let pairing_code_range = pairing_code_range::new(left, right, test_family.to_owned());

    let inserted_range = pairing_code_range::insert(pairing_code_range, &connection).unwrap();
    assert!(inserted_range.id() > 0);
    assert_eq!(test_family, inserted_range.family());
    assert_eq!(0, inserted_range.left());
    assert_eq!(999, inserted_range.right());

    let selected_range = pairing_code_range::select_by_id(inserted_range.id(), &connection);
    let selected_range = selected_range.unwrap().unwrap(); // unwrapping Result and Option
    assert_eq!(inserted_range, selected_range);
}

#[test]
fn can_delete_by_id() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let pairing_code_range = pairing_code_range::new(0, 100, fam.to_owned());
    let inserted_range = pairing_code_range::insert(pairing_code_range, &connection).unwrap();

    assert!(
        pairing_code_range::select_by_id(inserted_range.id(), &connection)
            .unwrap()
            .is_some()
    );
    pairing_code_range::delete_by_id(inserted_range.id(), &connection).unwrap();
    assert!(
        pairing_code_range::select_by_id(inserted_range.id(), &connection)
            .unwrap()
            .is_none()
    );
}

#[test]
fn can_delete_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let r1 =
        pairing_code_range::insert(pairing_code_range::new(0, 99, fam.to_owned()), &connection)
            .unwrap();
    let r2 = pairing_code_range::insert(
        pairing_code_range::new(100, 199, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let r3 = pairing_code_range::insert(
        pairing_code_range::new(200, 299, fam.to_owned()),
        &connection,
    )
    .unwrap();

    pairing_code_range::delete_family(&fam, &connection).unwrap();

    assert!(pairing_code_range::select_by_id(r1.id(), &connection)
        .unwrap()
        .is_none());
    assert!(pairing_code_range::select_by_id(r2.id(), &connection)
        .unwrap()
        .is_none());
    assert!(pairing_code_range::select_by_id(r3.id(), &connection)
        .unwrap()
        .is_none());
}

#[test]
fn can_find_range_on_the_left() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(pairing_code_range::new(0, 100, fam.to_owned()), &connection)
        .unwrap();
    let r2 = pairing_code_range::insert(
        pairing_code_range::new(200, 300, fam.to_owned()),
        &connection,
    )
    .unwrap();
    pairing_code_range::insert(
        pairing_code_range::new(400, 500, fam.to_owned()),
        &connection,
    )
    .unwrap();

    let range = pairing_code_range::select_first_to_the_left_of(350, &fam, &connection)
        .unwrap()
        .unwrap();

    assert_eq!(r2, range);
}

#[test]
fn does_not_find_range_on_the_left_when_it_does_not_exist() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range = pairing_code_range::select_first_to_the_left_of(99, &fam, &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn does_not_find_range_on_the_left_when_it_has_different_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range =
        pairing_code_range::select_first_to_the_left_of(300, "another_fam", &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn can_find_range_on_the_right() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(pairing_code_range::new(0, 100, fam.to_owned()), &connection)
        .unwrap();
    pairing_code_range::insert(
        pairing_code_range::new(200, 300, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let r3 = pairing_code_range::insert(
        pairing_code_range::new(400, 500, fam.to_owned()),
        &connection,
    )
    .unwrap();

    let range = pairing_code_range::select_first_to_the_right_of(350, &fam, &connection)
        .unwrap()
        .unwrap();

    assert_eq!(r3, range);
}

#[test]
fn does_not_find_range_on_the_right_when_it_does_not_exist() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range = pairing_code_range::select_first_to_the_right_of(201, &fam, &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn does_not_find_range_on_the_right_when_it_has_different_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range =
        pairing_code_range::select_first_to_the_right_of(50, "another_fam", &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn does_not_find_range_on_the_left_when_it_target_code_is_inside_the_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range = pairing_code_range::select_first_to_the_left_of(150, &fam, &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn does_not_find_range_on_the_right_when_it_target_code_is_inside_the_range() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let range = pairing_code_range::select_first_to_the_right_of(150, &fam, &connection).unwrap();
    assert!(range.is_none());
}

#[test]
fn cannot_insert_range_with_switched_sides() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let result = pairing_code_range::insert(
        pairing_code_range::new(300, 200, fam.to_owned()),
        &connection,
    );
    assert!(result.is_err());
}

#[test]
fn can_insert_range_with_equal_sides() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let result = pairing_code_range::insert(
        pairing_code_range::new(200, 200, fam.to_owned()),
        &connection,
    );
    assert!(result.is_ok());
}

#[test]
fn can_find_range_with_value_inside() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let r1 =
        pairing_code_range::insert(pairing_code_range::new(0, 100, fam.to_owned()), &connection)
            .unwrap();
    let r2 = pairing_code_range::insert(
        pairing_code_range::new(200, 300, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let r3 = pairing_code_range::insert(
        pairing_code_range::new(400, 500, fam.to_owned()),
        &connection,
    )
    .unwrap();

    let ri1 = pairing_code_range::select_first_range_with_value_inside(50, &fam, &connection)
        .unwrap()
        .unwrap();
    let ri2 = pairing_code_range::select_first_range_with_value_inside(250, &fam, &connection)
        .unwrap()
        .unwrap();
    let ri3 = pairing_code_range::select_first_range_with_value_inside(450, &fam, &connection)
        .unwrap()
        .unwrap();

    assert_eq!(r1, ri1);
    assert_eq!(r2, ri2);
    assert_eq!(r3, ri3);
}

#[test]
fn can_find_range_with_value_inside_when_val_is_on_either_side() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted = pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let ri1 = pairing_code_range::select_first_range_with_value_inside(100, &fam, &connection)
        .unwrap()
        .unwrap();
    let ri2 = pairing_code_range::select_first_range_with_value_inside(200, &fam, &connection)
        .unwrap()
        .unwrap();
    assert_eq!(inserted, ri1);
    assert_eq!(inserted, ri2);
}

#[test]
fn canot_find_range_with_value_inside_when_val_is_just_outside() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(
        pairing_code_range::new(100, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let ri1 =
        pairing_code_range::select_first_range_with_value_inside(99, &fam, &connection).unwrap();
    let ri2 =
        pairing_code_range::select_first_range_with_value_inside(201, &fam, &connection).unwrap();
    assert!(ri1.is_none());
    assert!(ri2.is_none());
}

#[test]
fn can_find_range_with_value_inside_when_range_has_single_value() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    let inserted = pairing_code_range::insert(
        pairing_code_range::new(200, 200, fam.to_owned()),
        &connection,
    )
    .unwrap();
    let ri = pairing_code_range::select_first_range_with_value_inside(200, &fam, &connection)
        .unwrap()
        .unwrap();
    assert_eq!(inserted, ri);
}

#[test]
fn range_with_value_inside_is_not_found_when_does_not_exist() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(pairing_code_range::new(0, 100, fam.to_owned()), &connection)
        .unwrap();
    pairing_code_range::insert(
        pairing_code_range::new(200, 300, fam.to_owned()),
        &connection,
    )
    .unwrap();
    pairing_code_range::insert(
        pairing_code_range::new(400, 500, fam.to_owned()),
        &connection,
    )
    .unwrap();

    let ri =
        pairing_code_range::select_first_range_with_value_inside(350, &fam, &connection).unwrap();
    assert!(ri.is_none());
}

#[test]
fn range_with_value_inside_is_not_found_when_it_has_different_family() {
    let fam = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam);
    let connection = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(pairing_code_range::new(0, 100, fam.to_owned()), &connection)
        .unwrap();

    let ri =
        pairing_code_range::select_first_range_with_value_inside(50, "another_fam", &connection)
            .unwrap();
    assert!(ri.is_none());
}

#[test]
fn select_family() {
    let fam1 = format!("{}{}", file!(), line!());
    let fam2 = format!("{}{}", file!(), line!());
    delete_ranges_with_family(&fam1);
    delete_ranges_with_family(&fam2);
    let conn = dbtesting_utils::testing_connection_for_client_user().unwrap();

    pairing_code_range::insert(pairing_code_range::new(10, 20, fam1.to_owned()), &conn).unwrap();
    pairing_code_range::insert(pairing_code_range::new(0, 9, fam1.to_owned()), &conn).unwrap();
    pairing_code_range::insert(pairing_code_range::new(21, 30, fam1.to_owned()), &conn).unwrap();
    pairing_code_range::insert(pairing_code_range::new(40, 50, fam2.to_owned()), &conn).unwrap();

    let codes = pairing_code_range::select_family(&fam1, &conn).unwrap();
    // Let's verify values and order
    assert_eq!(3, codes.len());
    assert_eq!(0, codes[0].left());
    assert_eq!(9, codes[0].right());
    assert_eq!(10, codes[1].left());
    assert_eq!(20, codes[1].right());
    assert_eq!(21, codes[2].left());
    assert_eq!(30, codes[2].right());
}
