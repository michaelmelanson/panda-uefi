#[lang = "eh_personality"]
fn eh_personality() {}

#[cfg(not(feature = "no_panic_handler"))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "Panic at {}:{}:{}:",
            location.file(),
            location.line(),
            location.column()
        );
        if let Some(message) = info.message() {
            log::error!("{}", message);
        }
    }

    // Give the user some time to read the message
    // if let Some(st) = unsafe { SYSTEM_TABLE.as_ref() } {
    //     st.boot_services().stall(10_000_000);
    // } else {
    let mut dummy = 0u64;
    // FIXME: May need different counter values in debug & release builds
    for i in 0..300_000_000 {
        unsafe {
            core::ptr::write_volatile(&mut dummy, i);
        }
    }
    // }

    // If running in QEMU, use the f4 exit port to signal the error and exit
    // use qemu_exit::QEMUExit;
    // let custom_exit_success = 3;
    // let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
    // qemu_exit_handle.exit_failure();

    // // If the system table is available, use UEFI's standard shutdown mechanism
    // if let Some(st) = unsafe { SYSTEM_TABLE.as_ref() } {
    //     use uefi::table::runtime::ResetType;
    //     st.runtime_services()
    //         .reset(ResetType::Shutdown, uefi::Status::ABORTED, None);
    // }

    // If we don't have any shutdown mechanism handy, the best we can do is loop

    log::error!("Could not shut down, please power off the system manually...");

    loop {
        unsafe {
            // Try to at least keep CPU from running at 100%
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

#[alloc_error_handler]
fn out_of_memory(layout: ::core::alloc::Layout) -> ! {
    panic!(
        "Ran out of free memory while trying to allocate {:#?}",
        layout
    );
}
