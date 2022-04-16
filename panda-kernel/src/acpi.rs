mod handler;

use acpi::{madt::Madt, platform::ProcessorInfo, sdt::Signature, InterruptModel};
use x86_64::PhysAddr;

use crate::memory;

use self::handler::AcpiMemoryHandler;

#[derive(Debug)]
pub enum AcpiError {
    NoRsdpProvided,

    AcpiError(::acpi::AcpiError),
}

impl From<::acpi::AcpiError> for AcpiError {
    fn from(error: ::acpi::AcpiError) -> Self {
        AcpiError::AcpiError(error)
    }
}

pub struct AcpiInitResult {
    pub interrupt_model: InterruptModel,
    pub processor_info: Option<ProcessorInfo>,
}

pub fn init(rsdp_address: Option<PhysAddr>) -> Result<AcpiInitResult, AcpiError> {
    match rsdp_address {
        Some(rsdp_address) => {
            let rsdp_address = memory::physical_to_virtual(rsdp_address);

            let acpi_tables = unsafe {
                ::acpi::AcpiTables::from_rsdp(AcpiMemoryHandler, rsdp_address.as_u64() as usize)?
            };

            let mut madt = None;

            acpi_tables.sdts.iter().for_each(|(sig, sdt)| {
                log::debug!("  SDT {sig} at {:X?}", sdt.physical_address);

                match sig {
                    &Signature::MADT => {
                        madt = Some(unsafe {
                            memory::physical_memory_ref::<Madt>(PhysAddr::new(
                                sdt.physical_address as u64,
                            ))
                        });
                    }

                    _ => {}
                }
            });

            let (interrupt_model, processor_info) = madt.unwrap().parse_interrupt_model().unwrap();

            Ok(AcpiInitResult {
                interrupt_model,
                processor_info,
            })
        }
        None => Err(AcpiError::NoRsdpProvided),
    }
}
