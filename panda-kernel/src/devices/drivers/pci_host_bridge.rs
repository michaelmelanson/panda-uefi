use alloc::vec::Vec;
use aml::resource::{AddressSpaceDescriptor, AddressSpaceResourceType, Resource};
use aml::{NamespaceLevel, resource::resource_descriptor_list};

use crate::pci::{PciDevice, PciRegister};
use crate::{acpi, pci, task};

pub fn init_from_acpi_level(acpi_level: NamespaceLevel) {
    task::start(discover_pci_host_bridge(acpi_level));
}

async fn discover_pci_host_bridge(acpi_level: NamespaceLevel) {
    log::info!("PCI host bridge driver started");
    
    let mut adr = None;
    let mut crs = None;

    for (name, value) in acpi_level.values {
        match name.as_str() {
            "_ADR" => adr = Some(value),
            "_CRS" => crs = Some(value),
            _ => {}
        } 
    }

    let mut segment = 0;
    if let Some(Ok(adr)) = adr.map(acpi::get) {
        match adr {
            aml::AmlValue::Integer(adr) => segment = adr as u16,
            _ => {}
        }
    }

    let mut bus_ranges = Vec::new();
    
    if let Some(Ok(crs)) = crs.map(acpi::get) {
        let resources = resource_descriptor_list(&crs);

        for resource in resources.unwrap() {
            match resource {
                Resource::AddressSpace(AddressSpaceDescriptor {
                    resource_type,
                    address_range: (low, high),
                    ..
                }) if resource_type == AddressSpaceResourceType::BusNumberRange => {
                    bus_ranges.push((low as u8, high as u8))
                }

                _ => {}
            }
        }

        for (from, to) in bus_ranges {
            for bus in from..=to {
                enumerate_pci_bus(segment, bus);
            }
        }
    } else {
        log::error!("Failed to get _CRS for PCI host bridge");
    }
    log::info!("Done enumerating PCI host bridge");
    
}

fn enumerate_pci_bus(segment: u16, bus: u8) {
    for device in 0..=32u8 {
        let function = 0;
        enumerate_pci_device(segment, bus, device, function);
    }
}

fn enumerate_pci_device(segment: u16, bus: u8, device: u8, function: u8) {
    if let Ok(vendor) = pci::read::<u16>(segment, bus, device, function, 0) {
        if vendor == 0xffff { return; }

        let pci_device = PciDevice::new(segment, bus, device, function);

        let header_type = pci_device.read::<u8>(PciRegister::HeaderType);
        crate::devices::drivers::init_by_pci(pci_device);

        if (header_type & 0x80) != 0 {
            for function in 1..8 {
                if let Ok(vendor) = pci::read::<u16>(segment, bus, device, function, 0) {
                    if vendor == 0xffff { continue; }
                    
                    let pci_device = PciDevice::new(segment, bus, device, function);
                    crate::devices::drivers::init_by_pci(pci_device);
                }
            }
        }
    }
}
