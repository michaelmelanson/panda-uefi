#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(start)]
#![feature(once_cell)]
#![feature(alloc_error_handler)]

mod alloc;
mod display;

use display::{FontSize, FontStyle, TextPart};
use panda_loader_lib::LoaderCarePackage;
use uart_16550::SerialPort;
extern crate core;

#[no_mangle]
pub fn _start(care_package: LoaderCarePackage) {
    let mut qemu_output = unsafe { SerialPort::new(0x3F8) };
    qemu_output.init();

    for c in "Hello, world!\n".chars() {
        qemu_output.send(c as u8);
    }

    let mut frame_buffer = care_package.frame_buffer;

    frame_buffer.draw_pixel((0, 0), (255, 0, 0));
    frame_buffer.draw_pixel((1, 0), (255, 0, 0));
    frame_buffer.draw_pixel((0, 1), (255, 0, 0));
    frame_buffer.draw_pixel((1, 1), (255, 0, 0));

    display::init(frame_buffer);
    display::write_text(TextPart("Panda\n", FontSize::Large, FontStyle::Bold));
}

#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
