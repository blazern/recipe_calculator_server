use log;
use std;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        InvalidUri(log::SetLoggerError);
    }
}
