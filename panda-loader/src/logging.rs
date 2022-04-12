use core::fmt::Write;

use uart_16550::SerialPort;
use uefi::table::Boot;

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
    if let Some(console) = unsafe { UEFI_CONSOLE } {
        let _ = unsafe { &mut *console }.write_fmt(args);
    } else if let Some(serial_port) = unsafe { &mut QEMU_OUTPUT } {
        let _ = serial_port.write_fmt(args);
    }
}

pub struct ConsoleLogger;

static mut QEMU_OUTPUT: Option<SerialPort> = None;
static mut UEFI_CONSOLE: Option<*mut uefi::proto::console::text::Output> = None;

pub fn init(system_table: &uefi::table::SystemTable<Boot>) -> Result<(), log::SetLoggerError> {
    let mut qemu_output = unsafe { SerialPort::new(0x3F8) };
    qemu_output.init();
    unsafe {
        QEMU_OUTPUT = Some(qemu_output);
    }

    let system_table = unsafe { system_table.unsafe_clone() };
    let console = system_table
        .boot_services()
        .locate_protocol::<uefi::proto::console::text::Output>()
        .unwrap();
    let console = &*console.clone();
    unsafe {
        UEFI_CONSOLE = Some(console.get());
    }

    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&ConsoleLogger)?;

    Ok(())
}

pub fn exit_boot_services() {
    unsafe {
        UEFI_CONSOLE = None;
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true // metadata.level() >= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            _print(format_args!(
                "{} - {} - {}\n",
                record.level(),
                record.target(),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}
