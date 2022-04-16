mod interrupts;

use acpi::InterruptModel;
use x2apic::lapic::{IpiDestMode, LocalApic, TimerDivide, TimerMode};
use x86_64::PhysAddr;

use crate::{
    interrupts::install_interrupt_handler,
    irq::interrupts::{lapic_error_handler, lapic_spurious_handler, lapic_timer_handler},
    memory,
};

static mut INTERRUPT_MODEL: Option<InterruptModel> = None;

const TIMER_VECTOR: usize = 0x20;
const ERROR_VECTOR: usize = 0x21;
const SPURIOUS_VECTOR: usize = 0x22;

#[derive(Debug)]
pub enum ApicError {
    NoApic,
    ApicError(&'static str),
}

pub fn lapic() -> Result<LocalApic, ApicError> {
    if let Some(InterruptModel::Apic(apic)) = unsafe { &INTERRUPT_MODEL } {
        let base_address = memory::physical_to_virtual(PhysAddr::new(apic.local_apic_address));

        let apic = x2apic::lapic::LocalApicBuilder::new()
            .set_xapic_base(base_address.as_u64())
            .ipi_destination_mode(IpiDestMode::Logical)
            .timer_vector(TIMER_VECTOR)
            .error_vector(ERROR_VECTOR)
            .spurious_vector(SPURIOUS_VECTOR)
            .build()
            .map_err(ApicError::ApicError)?;

        Ok(apic)
    } else {
        Err(ApicError::NoApic)
    }
}

pub fn init(interrupt_model: InterruptModel) {
    unsafe {
        INTERRUPT_MODEL = Some(interrupt_model);
    }

    if let Ok(mut lapic) = lapic() {
        install_interrupt_handler(TIMER_VECTOR, lapic_timer_handler);
        install_interrupt_handler(ERROR_VECTOR, lapic_error_handler);
        install_interrupt_handler(SPURIOUS_VECTOR, lapic_spurious_handler);

        unsafe {
            lapic.set_timer_mode(TimerMode::Periodic);
            lapic.set_timer_divide(TimerDivide::Div256);
            lapic.enable();
        }
    }
}

pub fn end_of_interrupt() {
    unsafe {
        match lapic() {
            Ok(mut lapic) => lapic.end_of_interrupt(),
            Err(err) => log::error!("End of interrupt failed: {:?}", err),
        }
    }
}
