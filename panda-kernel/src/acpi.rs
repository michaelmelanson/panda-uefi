mod acpi_memory_handler;
mod aml_handler;

use core::alloc::Allocator;

use ::acpi::AcpiTables;
use acpi::{madt::Madt, platform::ProcessorInfo, PciConfigRegions};
use alloc::{boxed::Box, string::String};
use aml::{AmlContext, AmlError, AmlHandle, AmlName, AmlValue, NamespaceLevel};
use conquer_once::spin::OnceCell;
use eisaid::decode_eisa_id;
use linked_list_allocator::LockedHeap;
use spin::RwLock;
use x86_64::PhysAddr;

use crate::{
    acpi::aml_handler::AmlHandler,
    devices::drivers,
    irq,
    memory::{self, physical_memory_ref, GLOBAL_ALLOCATOR},
    pci,
};

use self::acpi_memory_handler::AcpiMemoryHandler;

static mut AML_CONTEXT: OnceCell<RwLock<AmlContext>> = OnceCell::uninit();

#[derive(Debug)]
pub enum AcpiError {
    NotInitialized,
    NoRsdpProvided,

    AcpiError(::acpi::AcpiError),
    AmlError(::aml::AmlError),
}

impl From<::acpi::AcpiError> for AcpiError {
    fn from(error: ::acpi::AcpiError) -> Self {
        AcpiError::AcpiError(error)
    }
}

impl From<::aml::AmlError> for AcpiError {
    fn from(error: ::aml::AmlError) -> Self {
        AcpiError::AmlError(error)
    }
}

pub struct AcpiInitResult<'a, A: Allocator> {
    pub processor_info: Option<ProcessorInfo<'a, A>>,
}

pub fn init<'a>(
    rsdp_address: Option<PhysAddr>,
) -> Result<AcpiInitResult<'a, LockedHeap>, AcpiError> {
    let rsdp_address = match rsdp_address {
        Some(rsdp_address) => rsdp_address,
        None => return Err(AcpiError::NoRsdpProvided),
    };

    let rsdp_address = memory::physical_to_virtual(rsdp_address);

    let acpi_tables =
        unsafe { AcpiTables::from_rsdp(AcpiMemoryHandler, rsdp_address.as_u64() as usize)? };

    let madt = unsafe {
        acpi_tables
            .find_table::<Madt>()
            .map(|sdt| physical_memory_ref::<Madt>(PhysAddr::new(sdt.physical_start() as u64)))
    };

    let pci_config_regions = PciConfigRegions::new_in(&acpi_tables, &GLOBAL_ALLOCATOR)?;
    pci::init(pci_config_regions);

    let (interrupt_model, processor_info) = madt
        .unwrap()
        .parse_interrupt_model_in(&GLOBAL_ALLOCATOR)
        .unwrap();
    irq::init(interrupt_model);

    let aml_context = aml::AmlContext::new(Box::new(AmlHandler::new()), aml::DebugVerbosity::All);

    unsafe {
        AML_CONTEXT.init_once(|| RwLock::new(aml_context));
    }

    // log::info!("SDTs: {:?}", acpi_tables..ssdts().map(|table| table.).keys());
    if let Ok(sdt) = acpi_tables.dsdt() {
        log::info!("Found SSDT at 0x{:X}", sdt.address);
        let ptr = memory::physical_to_virtual(PhysAddr::new(sdt.address as u64));
        let ptr = ptr.as_mut_ptr() as *mut u8;
        let stream = unsafe { core::slice::from_raw_parts(ptr, sdt.length as usize) };
        unsafe { AML_CONTEXT.get() }
            .unwrap()
            .write()
            .parse_table(stream)
            .unwrap();

        log::info!("Traversing SSDT tables");

        // let mut entries = Vec::new();

        unsafe { AML_CONTEXT.get() }
            .unwrap()
            .write()
            .initialize_objects()?;

        let aml_context = unsafe { AML_CONTEXT.get() }.unwrap().read();
        aml_context.namespace.clone().traverse(|name, level| {
            match enumerate_acpi_device(&aml_context, name, level) {
                EnumerateAcpiDeviceBehaviour::TraverseChildren => Ok(true),
                EnumerateAcpiDeviceBehaviour::NoTraverseChildren => Ok(false),
            }
        })?;
    }

    Ok(AcpiInitResult { processor_info })
}

pub fn to_handle(aml_name: &AmlName) -> Result<AmlHandle, AcpiError> {
    let aml_context = unsafe { AML_CONTEXT.get() }
        .ok_or(AcpiError::NotInitialized)?
        .read();

    let handle = aml_context.namespace.get_handle(aml_name)?;
    Ok(handle)
}

pub fn get_key(context: &AmlContext, aml_name: &AmlName, key: &str) -> Result<AmlValue, AmlError> {
    let child_aml_name = AmlName::from_str(key)?;
    let resolved_name = child_aml_name.resolve(aml_name)?;

    let value = context.namespace.get_by_path(&resolved_name)?;
    Ok(value.clone())
}

pub fn get(handle: AmlHandle) -> Result<AmlValue, AcpiError> {
    let aml_context = unsafe { AML_CONTEXT.get() }
        .ok_or(AcpiError::NotInitialized)?
        .read();
    let value = aml_context.namespace.get(handle)?;
    Ok(value.clone())
}

pub fn get_as_string(handle: AmlHandle) -> Result<String, AcpiError> {
    let aml_context = unsafe { AML_CONTEXT.get() }
        .ok_or(AcpiError::NotInitialized)?
        .read();
    let value = aml_context.namespace.get(handle)?;
    let string = value.as_string(&aml_context)?;
    Ok(string)
}

pub fn get_as_integer(handle: AmlHandle) -> Result<u64, AcpiError> {
    let aml_context = unsafe { AML_CONTEXT.get() }
        .ok_or(AcpiError::NotInitialized)?
        .read();
    let value = aml_context.namespace.get(handle)?;
    let integer = value.as_integer(&aml_context)?;
    Ok(integer)
}

pub enum EnumerateAcpiDeviceBehaviour {
    TraverseChildren,
    NoTraverseChildren,
}

pub fn enumerate_acpi_device(
    aml_context: &AmlContext,
    aml_name: &AmlName,
    level: &NamespaceLevel,
) -> EnumerateAcpiDeviceBehaviour {
    let behaviour = EnumerateAcpiDeviceBehaviour::TraverseChildren;

    match level.typ {
        aml::LevelType::Device => {
            let mut hid = None;

            for (name, handle) in &level.values {
                match name.as_str() {
                    "_HID" => {
                        if let Ok(value) = aml_context.namespace.get(*handle) {
                            let decoded_value = decode_hid_value(value);
                            hid = decoded_value;
                        }
                    }

                    _ => {}
                }
            }

            if let Some(hid) = hid {
                log::info!(" -> Device {aml_name} (HID {hid})");
                drivers::init_by_acpi(hid, level.clone());
            }
        }
        aml::LevelType::Processor => {
            let mut hid = None;

            for (name, handle) in &level.values {
                match name.as_str() {
                    "_HID" => {
                        if let Ok(value) = aml_context.namespace.get(*handle) {
                            let decoded_value = decode_hid_value(value);
                            hid = decoded_value;
                        }
                    }

                    _ => {}
                }
            }

            log::info!(" -> Processor #{aml_name} (HID {hid:?})");
        }
        _ => log::debug!(" -> Ignoring: #{aml_name}"),
    }

    behaviour
}

fn decode_hid_value(value: &AmlValue) -> Option<String> {
    match value {
        AmlValue::String(value) => Some(value.clone()),
        AmlValue::Integer(value) => Some(decode_eisa_id(*value as u32)),
        _ => None,
    }
}
