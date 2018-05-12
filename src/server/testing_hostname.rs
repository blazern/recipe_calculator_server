use std::sync::Mutex;
use std::sync::MutexGuard;

lazy_static! {
    static ref HOSTNAME: Mutex<String> = Mutex::new("127.0.0.1:3000".into());
}

#[cfg(test)]
pub fn get_hostname() -> MutexGuard<'static, String> {
    HOSTNAME.lock().unwrap()
}