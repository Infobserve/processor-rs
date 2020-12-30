use log::{LevelFilter, SetLoggerError};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::{RollingFileAppender, LogFile, policy::Policy};
use log4rs::config::{Appender, Config, Root};
use log4rs::Handle;
use anyhow::Result;

const FILE_ROLL_BYTE_THRESHOLD: u64 = 2_500_000;
const LOGFILE_PATH: &str = "logs/processor.log";

#[allow(dead_code)]
pub struct Logger {
    handle: Handle
}

#[derive(Debug)]
struct SizeRotatePolicy;

impl Policy for SizeRotatePolicy {
    fn process(&self, log: &mut LogFile<'_>) -> Result<()> {
        if log.len_estimate() > FILE_ROLL_BYTE_THRESHOLD {
            log.roll();
        }
        Ok(())
    }
}

pub fn init() -> Result<Logger, SetLoggerError> {

    let console = ConsoleAppender::builder().build();
    let pol = SizeRotatePolicy;

    let rollfile = RollingFileAppender::builder().build(LOGFILE_PATH, Box::new(pol)).unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("rollfile", Box::new(rollfile)))
        .logger(log4rs::config::Logger::builder().build("app::backend::db", LevelFilter::Debug))
        .logger(log4rs::config::Logger::builder()
            .appender("rollfile")
            .additive(false)
            .build("app::rollfile", LevelFilter::Debug))
        .build(Root::builder().appender("console").build(LevelFilter::Info))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();

    Ok(Logger { handle })
}
