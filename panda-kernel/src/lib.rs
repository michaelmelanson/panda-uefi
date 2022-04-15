#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(start)]
#![feature(once_cell)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
extern crate alloc;

#[macro_use]
mod console;
mod display;
mod interrupts;
mod logger;
mod memory;

pub use crate::console::_print;

use display::{FontSize, FontStyle, TextPart};
use logger::LoggerError;
use memory::MemoryError;
use panda_loader_lib::{KernelEntryFn, LoaderCarePackage, LoaderCarePackageError};

extern crate core;

#[no_mangle]
pub extern "win64" fn _start(care_package: &LoaderCarePackage) {
    kernel_main(care_package).expect("Kernel panic");
}

#[derive(Debug)]
pub enum KernelError {
    LoggerError(LoggerError),
    LoaderCarePackageError(LoaderCarePackageError),
    MemoryError(MemoryError),
}

impl From<LoggerError> for KernelError {
    fn from(error: LoggerError) -> Self {
        KernelError::LoggerError(error)
    }
}

impl From<LoaderCarePackageError> for KernelError {
    fn from(error: LoaderCarePackageError) -> Self {
        KernelError::LoaderCarePackageError(error)
    }
}

impl From<MemoryError> for KernelError {
    fn from(error: MemoryError) -> Self {
        KernelError::MemoryError(error)
    }
}

fn kernel_main(care_package: &LoaderCarePackage) -> Result<(), KernelError> {
    console::init();
    logger::init()?;

    care_package.validate()?;

    interrupts::init();
    memory::init(
        &care_package.memory_map,
        care_package.phys_memory_virt_offset,
    )?;

    display::init(care_package.frame_buffer.clone());
    display::write_text(TextPart("Panda\n", FontSize::Large, FontStyle::Bold));
    display::write_text(TextPart(
        "Looks like everything's working!\n",
        FontSize::Regular,
        FontStyle::Regular,
    ));

    Ok(())
}

// check that it's compatible with the entry point
#[allow(dead_code)]
const ENTRY_FN: KernelEntryFn = _start;

#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
