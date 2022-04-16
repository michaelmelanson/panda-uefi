use log::Level;

#[derive(Debug)]
pub enum LoggerError {
    SetLoggerError(log::SetLoggerError),
}

impl From<log::SetLoggerError> for LoggerError {
    fn from(error: log::SetLoggerError) -> Self {
        LoggerError::SetLoggerError(error)
    }
}

pub fn init() -> Result<(), LoggerError> {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&ConsoleLogger)?;
    Ok(())
}

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} - {} - {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
