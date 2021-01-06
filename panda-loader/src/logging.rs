use crate::display;

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logging::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    display::DISPLAY
        .get()
        .unwrap()
        .lock()
        .write_fmt(args)
        .unwrap();
}

pub struct ConsoleLogger;

static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&CONSOLE_LOGGER)
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
