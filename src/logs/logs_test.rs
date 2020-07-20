use crate::logs::init_logs;
use crate::logs::logs_error::Error;
use crate::logs::logs_error::ErrorKind;

use std::fs::create_dir;
use std::io::ErrorKind as IOErrorKind;

const FILE_PREFIX: &str = "/tmp/logs_file_for_tests";

#[test]
fn init_logs_when_file_does_not_exist() {
    let filename = FILE_PREFIX.to_string() + &time::precise_time_ns().to_string();

    let result = init_logs(&filename);

    match result {
        Err(Error(ErrorKind::Io(ioerr), _)) => {
            assert_eq!(IOErrorKind::NotFound, ioerr.kind());
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Error expected"),
    }
}

#[test]
fn init_logs_when_passed_path_is_dir() {
    let dirname = FILE_PREFIX.to_string() + &time::precise_time_ns().to_string();
    create_dir(&dirname).unwrap();

    let result = init_logs(&dirname);

    match result {
        Err(Error(ErrorKind::Io(ioerr), _)) => {
            assert_eq!(IOErrorKind::InvalidInput, ioerr.kind());
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
        Ok(_) => panic!("Error expected"),
    }
}
