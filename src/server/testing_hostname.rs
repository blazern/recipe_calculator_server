use std::sync::Mutex;
use std::sync::MutexGuard;

lazy_static! {
    static ref HOSTNAME: Mutex<String> = Mutex::new("127.0.0.1:3000".into());
    static ref SPARE_HOSTNAME1: Mutex<String> = Mutex::new("127.0.0.1:3001".into());
    static ref SPARE_HOSTNAME2: Mutex<String> = Mutex::new("127.0.0.1:3002".into());
    static ref SPARE_HOSTNAME3: Mutex<String> = Mutex::new("127.0.0.1:3003".into());
    static ref SPARE_HOSTNAME4: Mutex<String> = Mutex::new("127.0.0.1:3004".into());
    static ref SPARE_HOSTNAME5: Mutex<String> = Mutex::new("127.0.0.1:3005".into());
}

#[cfg(test)]
pub fn get_hostname() -> MutexGuard<'static, String> {
    HOSTNAME.lock().unwrap()
}

#[cfg(test)]
pub fn get_spare_hostname1() -> MutexGuard<'static, String> {
    SPARE_HOSTNAME1.lock().unwrap()
}

#[cfg(test)]
pub fn get_spare_hostname2() -> MutexGuard<'static, String> {
    SPARE_HOSTNAME2.lock().unwrap()
}

#[cfg(test)]
pub fn get_spare_hostname3() -> MutexGuard<'static, String> {
    SPARE_HOSTNAME3.lock().unwrap()
}

#[cfg(test)]
pub fn get_spare_hostname4() -> MutexGuard<'static, String> {
    SPARE_HOSTNAME4.lock().unwrap()
}

#[cfg(test)]
pub fn get_spare_hostname5() -> MutexGuard<'static, String> {
    SPARE_HOSTNAME5.lock().unwrap()
}
