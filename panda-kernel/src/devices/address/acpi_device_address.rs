use alloc::string::String;
use aml::AmlName;

use crate::acpi;

use super::pci_device_address::PciDeviceAddress;

#[derive(Debug)]
pub struct AcpiDeviceAddress(AmlName);

impl AcpiDeviceAddress {
    pub fn new(name: AmlName) -> Self {
        AcpiDeviceAddress(name)
    }

    pub fn display_name(&self) -> Option<String> {
        let name_path = match AmlName::from_str("_STR").and_then(|name| name.resolve(&self.0)) {
            Ok(name_path) => {
                acpi::to_handle(&name_path).expect("failed to convert AML name to handle")
            }
            _ => return None,
        };

        let name_string = match acpi::get_as_string(name_path) {
            Ok(name_string) => name_string,
            _ => return None,
        };

        Some(name_string)
    }

    pub fn pci_address(&self) -> Option<PciDeviceAddress> {
        let address_path = match AmlName::from_str("_ADR").and_then(|name| name.resolve(&self.0)) {
            Ok(address_path) => acpi::to_handle(&address_path)
                .expect("failed to convert PCI address path to ACPI handle"),
            _ => return None,
        };

        let address = match acpi::get_as_integer(address_path) {
            Ok(address) => address,
            _ => return None,
        };

        let device = (address >> 16) as u8;
        let function = (address & 0xFFFF) as u8;

        Some(PciDeviceAddress::new(0x55, 0x42, device, function))
    }
}
