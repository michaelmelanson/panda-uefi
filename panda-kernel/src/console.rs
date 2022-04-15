use core::fmt::Write;

use alloc::format;
use uart_16550::SerialPort;

use crate::display::{self, FontSize, FontStyle, TextPart};

static mut QEMU_OUTPUT: Option<SerialPort> = None;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    if let Some(serial_port) = unsafe { &mut QEMU_OUTPUT } {
        let _ = serial_port.write_fmt(args);
    }
    display::write_text(TextPart(
        format!("{}", args).as_str(),
        FontSize::Regular,
        FontStyle::Regular,
    ));
}

pub fn init() {
    unsafe {
        let mut serial_port = SerialPort::new(0x3F8);
        serial_port.init();
        QEMU_OUTPUT = Some(serial_port);
    }
}
