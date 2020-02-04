use std::time::SystemTime;
use std::time::SystemTimeError;
use std::time::UNIX_EPOCH;

/// Used in tests
pub trait NowSource {
    fn now_secs(&self) -> Result<i64, SystemTimeError>;
}
#[derive(Default, Debug)]
pub struct DefaultNowSource;
impl NowSource for DefaultNowSource {
    fn now_secs(&self) -> Result<i64, SystemTimeError> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
    }
}
