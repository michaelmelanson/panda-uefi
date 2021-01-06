#![no_std]
#![feature(lang_items)]
extern crate core;

use panda_loader_lib::LoaderCarePackage;

#[no_mangle]
pub extern "C" fn kernel_main(_package: *const LoaderCarePackage) -> ! {
    loop {}
}

#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
