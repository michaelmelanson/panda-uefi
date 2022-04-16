mod page_fault;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::irq::end_of_interrupt;

use self::page_fault::page_fault_handler;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);

        IDT[0x20].set_handler_fn(timer_interrupt_handler);
        IDT[0x21].set_handler_fn(lapic_error_handler);
        IDT.load()
    }
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}\nError code: {:X}",
        stack_frame, error_code
    );
}

pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError code: {:X}",
        stack_frame, error_code
    );
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    log::info!("Timer interrupt");
    end_of_interrupt();
}
pub extern "x86-interrupt" fn lapic_error_handler(_stack_frame: InterruptStackFrame) {
    log::info!("LAPIC error");
    end_of_interrupt();
}
