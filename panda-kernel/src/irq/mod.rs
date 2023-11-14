mod interrupts;

use core::ops::Range;

use acpi::InterruptModel;
use linked_list_allocator::LockedHeap;
use x2apic::{
    ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry},
    lapic::{IpiDestMode, LocalApic, TimerDivide, TimerMode},
};
use x86_64::{structures::idt::HandlerFunc, PhysAddr};

use crate::{
    interrupts::install_interrupt_handler,
    irq::interrupts::{lapic_error_handler, lapic_spurious_handler, lapic_timer_handler},
    memory,
};

static mut INTERRUPT_MODEL: Option<InterruptModel<'static, LockedHeap>> = None;

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
            .timer_mode(TimerMode::OneShot)
            .timer_divide(TimerDivide::Div256)
            .build()
            .map_err(ApicError::ApicError)?;

        Ok(apic)
    } else {
        Err(ApicError::NoApic)
    }
}

unsafe fn ioapic_indexes() -> Result<Range<usize>, ApicError> {
    if let Some(InterruptModel::Apic(apic)) = &INTERRUPT_MODEL {
        Ok(0..apic.io_apics.len())
    } else {
        Err(ApicError::NoApic)
    }
}
unsafe fn ioapic(index: usize) -> Result<IoApic, ApicError> {
    match &INTERRUPT_MODEL {
        Some(InterruptModel::Apic(apic)) => {
            let base_address =
                memory::physical_to_virtual(PhysAddr::new(apic.io_apics[index].address as u64));

            let ioapic = IoApic::new(base_address.as_u64());
            Ok(ioapic)
        }
        _ => todo!(),
    }
}

pub fn init(interrupt_model: InterruptModel<'static, LockedHeap>) {
    unsafe {
        INTERRUPT_MODEL = Some(interrupt_model);
    }

    if let Ok(mut lapic) = lapic() {
        install_interrupt_handler(TIMER_VECTOR, lapic_timer_handler);
        install_interrupt_handler(ERROR_VECTOR, lapic_error_handler);
        install_interrupt_handler(SPURIOUS_VECTOR, lapic_spurious_handler);

        unsafe {
            lapic.enable();
        }
    }
}

pub fn configure_irq(
    irq: u8,
    destination: u8,
    vector: u8,
    flags: IrqFlags,
    handler: HandlerFunc,
) -> Result<(), ApicError> {
    for ioapic_index in unsafe { ioapic_indexes()? } {
        if let Ok(mut ioapic) = unsafe { ioapic(ioapic_index) } {
            let mut redirection_entry = RedirectionTableEntry::default();
            redirection_entry.set_vector(vector);
            redirection_entry.set_dest(destination);
            redirection_entry.set_flags(flags);
            redirection_entry.set_mode(IrqMode::Fixed);

            unsafe {
                ioapic.set_table_entry(irq, redirection_entry);
            }

            install_interrupt_handler(vector as usize, handler);
        }
    }

    Ok(())
}

pub fn enable_irq(ioapic_index: usize, irq: u8) {
    unsafe {
        if let Ok(mut ioapic) = ioapic(ioapic_index) {
            ioapic.enable_irq(irq);
        } else {
            log::warn!("Could not enable IRQ {} on IO APIC {}", irq, ioapic_index);
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
