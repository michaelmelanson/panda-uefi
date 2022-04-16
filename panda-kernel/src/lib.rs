#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(start)]
#![feature(once_cell)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
extern crate alloc;

mod acpi;
#[macro_use]
mod console;
mod display;
mod error;
mod interrupts;
mod irq;
mod logger;
mod memory;
mod panic;

pub use crate::console::_print;

use display::{FontSize, FontStyle, TextPart};
use error::KernelError;
use panda_loader_lib::{KernelEntryFn, LoaderCarePackage};

extern crate core;

#[no_mangle]
pub extern "win64" fn _start(care_package: &LoaderCarePackage) {
    kernel_init(care_package).expect("Failed to initialize kernel");
    kernel_main().expect("Kernel panic");
}

fn kernel_init(care_package: &LoaderCarePackage) -> Result<(), KernelError> {
    console::init();
    logger::init()?;

    care_package.validate()?;

    interrupts::init();
    memory::init(
        &care_package.memory_map,
        care_package.phys_memory_virt_offset,
    )?;
    display::init(care_package.frame_buffer.clone());

    let acpi = acpi::init(care_package.rsdp_address)?;
    irq::init(acpi.interrupt_model);
    Ok(())
}

fn kernel_main() -> Result<(), KernelError> {
    x86_64::instructions::interrupts::enable();

    display::write_text(TextPart("Panda OS\n", FontSize::Large, FontStyle::Bold));
    log::info!("Looks like everything's working!");

    loop {
        x86_64::instructions::hlt();
        log::debug!("HLT interrupted");
    }
}

// check that it's compatible with the entry point
#[allow(dead_code)]
const ENTRY_FN: KernelEntryFn = _start;
