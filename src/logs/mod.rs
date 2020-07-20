pub mod logs_error;

use std::fs;
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::Appender;
use log4rs::config::Config;
use log4rs::config::Root;
use log4rs::encode::pattern::PatternEncoder;

use logs_error::Error;

const LOG_FILE_SIZE_LIMIT: u64 = 1024 * 1024 * 10;

pub fn init_logs(target_logs_file: &str) -> Result<(), Error> {
    let file_metadata = fs::metadata(target_logs_file);
    let file_metadata = match file_metadata {
        Ok(file_metadata) => file_metadata,
        Err(ioerr) => {
            assert_eq!(IOErrorKind::NotFound, ioerr.kind());
            let msg = "Logs file not found. You have to precreate the logs ".to_owned()
                + "file if it doesn't exist yet so that it would be impossible to "
                + "lose logs because of some invalid logs file path";
            return Err(IOError::new(
                IOErrorKind::NotFound,
                format!("{}. Received path: {}", msg, target_logs_file),
            )
            .into());
        }
    };
    if !file_metadata.is_file() {
        let msg = "Logs file is not a file. ".to_owned()
            + "Did you forget to precreate the file when starting a docker container? "
            + "Docker creates directories for volumes when specified volume "
            + "doesn't exist";
        return Err(IOError::new(
            IOErrorKind::InvalidInput,
            format!("{}. Path: {}", msg, target_logs_file),
        )
        .into());
    }

    let stdout = ConsoleAppender::builder().build();

    let rollingfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(
            target_logs_file,
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(LOG_FILE_SIZE_LIMIT)),
                Box::new(DeleteRoller::new()),
            )),
        )
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("rollingfile", Box::new(rollingfile)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("rollingfile")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config)?;

    Ok(())
}

#[cfg(test)]
#[path = "./logs_test.rs"]
mod logs_test;
