use alloc::string::String;
use aml::NamespaceLevel;

use crate::{acpi::EnumerateAcpiDeviceBehaviour, pci::{PciDevice, PciRegister}};

pub mod pc_keyboard;
pub mod pci_host_bridge;
pub mod ahci_controller;

pub fn init_by_acpi(hid: String, level: NamespaceLevel) -> EnumerateAcpiDeviceBehaviour {
    let mut behaviour = EnumerateAcpiDeviceBehaviour::TraverseChildren;

    match hid.as_str() {
        "PNP0A03" | "PNP0A08" => {
            pci_host_bridge::init_from_acpi_level(level);
            behaviour = EnumerateAcpiDeviceBehaviour::NoTraverseChildren;
        }

        "PNP0303" => pc_keyboard::init_from_acpi_level(level),
        _ => {}
    }

    behaviour
}

pub fn init_by_pci(pci_device: PciDevice) {
    let vendor = pci_device.read::<u16>(PciRegister::VendorId);
    let class = pci_device.read::<u8>(PciRegister::ClassCode);
    let subclass = pci_device.read::<u8>(PciRegister::SubclassCode);
    let device_id = pci_device.read::<u16>(PciRegister::DeviceId);
    let subsystem_id = pci_device.read::<u16>(PciRegister::SubsystemId);
    let prog_if = pci_device.read::<u8>(PciRegister::ProgIf);

    match (class, subclass, prog_if, vendor, device_id, subsystem_id) {
        // (0x8086, 0x2918, _) => {
        //     log::info!("  Found ISA host bridge");
        //     isa_host_bridge::init_from_pci_device(pci_device)
        // }

        (0x01, 0x06, 0x01, _, _, _) => ahci_controller::init_from_pci_device(pci_device),

        _ => {
            log::info!("  Unknown PCI device  (class {class:X}, subclass {subclass:X}, vendor {vendor:X}, device id {device_id:X}, subsystem id {subsystem_id:X}), prog if {prog_if:X}");
        }
    }
}
