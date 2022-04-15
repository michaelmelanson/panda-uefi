use x86_64::{
    registers::control::Cr2,
    structures::{
        idt::{InterruptStackFrame, PageFaultErrorCode},
        paging::Page,
    },
};

use crate::memory;

pub extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address = Cr2::read();
    let page = Page::containing_address(address);

    match address.as_u64() {
        0xD0000000..=0xE0000000 => {
            let frame = memory::allocate_frame().expect("Failed to allocate frame");
            memory::map_page_to_frame(page, frame).expect("Failed to map page to frame");
        }
        _ => {
            println!("EXCEPTION: INVALID PAGE FAULT");
            println!("Error code: {:?}", error_code);
            println!("Faulting address: {:?}", address);
            panic!("Invalid page fault");
        }
    }
}
