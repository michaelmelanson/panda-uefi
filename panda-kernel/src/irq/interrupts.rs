use x86_64::structures::idt::InterruptStackFrame;

use crate::irq::end_of_interrupt;
pub extern "x86-interrupt" fn lapic_timer_handler(_stack_frame: InterruptStackFrame) {
    end_of_interrupt();
}

pub extern "x86-interrupt" fn lapic_error_handler(_stack_frame: InterruptStackFrame) {
    log::info!("LAPIC error");
    end_of_interrupt();
}

pub extern "x86-interrupt" fn lapic_spurious_handler(_stack_frame: InterruptStackFrame) {
    log::info!("LAPIC sent spurious IRQ");
    end_of_interrupt();
}
