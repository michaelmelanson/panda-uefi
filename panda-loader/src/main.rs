#![no_std]
#![no_main]
#![feature(abi_efiapi)]
extern crate alloc;

mod display;
mod logging;

use alloc::{string::ToString, vec::Vec};
use display::{FontSize, FontStyle, TextPart};
use uefi::prelude::*;

#[entry]
fn uefi_start(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize utils");

    // reset console before doing anything else
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset output buffer");

    display::init(&system_table);

    display::write_text(TextPart(
        "Panda\n".to_string(),
        FontSize::Large,
        FontStyle::Bold,
    ));

    logging::init().unwrap();

    // Print out UEFI revision number
    {
        let rev = system_table.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());

        log::info!("Booted by UEFI {}.{}!", major, minor);
    }

    log::info!("Exiting boot services...");
    let mut mmap_buf = Vec::new();
    mmap_buf.resize(system_table.boot_services().memory_map_size(), 0);

    system_table
        .exit_boot_services(handle, &mut mmap_buf)
        .expect_success("Could not exit boot services");

    loop {}
}
