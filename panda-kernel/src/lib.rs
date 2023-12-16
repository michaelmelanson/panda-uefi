#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(lang_items)]
#![feature(start)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(async_closure)]
#![feature(link_llvm_intrinsics)]
extern crate alloc;

mod acpi;
#[macro_use]
mod console;
mod devices;
mod display;
mod error;
mod interrupts;
mod irq;
mod logger;
mod memory;
mod panic;
mod pci;
mod task;
mod util;

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
    interrupts::init();

    care_package.validate()?;

    memory::init(
        &care_package.memory_map,
        care_package.phys_memory_virt_offset,
    )?;
    display::init(care_package.frame_buffer.clone());
    task::init();

    let _ = acpi::init(care_package.rsdp_address)?;

    Ok(())
}

fn kernel_main() -> Result<(), KernelError> {
    x86_64::instructions::interrupts::enable();

    display::write_text(TextPart("Panda OS\n", FontSize::Large, FontStyle::Bold));
    log::info!("Looks like everything's working!");

    loop {
        task::step();

        if task::is_queue_empty() {
            x86_64::instructions::interrupts::enable_and_hlt();
            x86_64::instructions::interrupts::disable();
            log::info!("HLT interrupted");
        }
    }
}

// check that it's compatible with the entry point
#[allow(dead_code)]
const ENTRY_FN: KernelEntryFn = _start;
