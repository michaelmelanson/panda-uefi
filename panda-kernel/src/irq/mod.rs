use acpi::InterruptModel;
use x2apic::lapic::{IpiDestMode, LocalApic, TimerDivide, TimerMode};
use x86_64::PhysAddr;

use crate::memory;

static mut INTERRUPT_MODEL: Option<InterruptModel> = None;

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
            .timer_vector(0x20)
            .error_vector(0x21)
            .spurious_vector(0x22)
            .build()
            .map_err(ApicError::ApicError)?;

        Ok(apic)
    } else {
        Err(ApicError::NoApic)
    }
}

pub fn init(interrupt_model: InterruptModel) {
    log::info!("Interrupt model: {:?}", interrupt_model);

    unsafe {
        INTERRUPT_MODEL = Some(interrupt_model);

        let mut lapic = lapic().unwrap();
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_divide(TimerDivide::Div256);
        lapic.enable();
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
