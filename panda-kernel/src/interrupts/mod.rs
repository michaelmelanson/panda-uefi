mod page_fault;

use x86_64::structures::idt::InterruptDescriptorTable;

use self::page_fault::page_fault_handler;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.load()
    }
}
