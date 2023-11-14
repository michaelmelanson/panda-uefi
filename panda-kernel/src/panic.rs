#[cfg(not(test))]
#[panic_handler]
pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", _info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
