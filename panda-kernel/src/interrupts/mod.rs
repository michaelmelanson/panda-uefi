mod page_fault;

use x86_64::structures::idt::{HandlerFunc, InterruptDescriptorTable, InterruptStackFrame};

use self::page_fault::page_fault_handler;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);

        // map all IRQs to a default handler
        for i in 0x20..0xFF {
            IDT[i].set_handler_fn(default_irq_handler);
        }

        IDT.load()
    }
}

pub fn install_interrupt_handler(vector: usize, handler: HandlerFunc) {
    unsafe {
        IDT[vector].set_handler_fn(handler);
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

pub extern "x86-interrupt" fn default_irq_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: UNEXPECTED IRQ\n{:#?}", stack_frame);
}
