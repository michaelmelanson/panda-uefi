use alloc::{
    string::String,
    sync::Arc,
    vec::Vec,
};
use aml::AmlName;
use crossbeam_queue::SegQueue;

use crate::acpi;

pub mod drivers;

#[derive(Debug)]
pub struct AcpiDeviceAddress(AmlName);

impl AcpiDeviceAddress {
    pub fn new(name: AmlName) -> Self {
        AcpiDeviceAddress(name)
    }

    pub fn display_name(&self) -> Option<String> {
        let name_path = match AmlName::from_str("_STR").and_then(|name| name.resolve(&self.0)) {
            Ok(name_path) => name_path,
            _ => return None,
        };
        let name_string = match acpi::get_as_string(&name_path) {
            Ok(name_string) => name_string,
            _ => return None,
        };

        Some(name_string)
    }

    pub fn pci_address(&self) -> Option<PciDeviceAddress> {
        let address_path = match AmlName::from_str("_ADR").and_then(|name| name.resolve(&self.0)) {
            Ok(address_path) => address_path,
            _ => return None,
        };

        let address = match acpi::get_as_integer(&address_path) {
            Ok(address) => address,
            _ => return None,
        };

        let device = (address >> 16) as u8;
        let function = (address & 0xFFFF) as u8;

        Some(PciDeviceAddress {
            segment: 0x55,
            bus: 0x42,
            device,
            function,
        })
    }
}

#[derive(Debug)]
pub struct PciDeviceAddress {
    segment: u8,
    bus: u8,
    device: u8,
    function: u8,
}

pub struct Driver<Message> {
    pub inbox: Arc<SegQueue<Message>>,
}

impl<M> core::fmt::Debug for Driver<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Driver")
    }
}

impl<Message> Driver<Message> {
    pub fn new() -> Self {
        let inbox = Arc::new(SegQueue::new());
        Self { inbox }
    }
}

pub enum StandardDeviceMessage {
    Terminate,
}

pub enum KeyboardDeviceMessage {

}

pub enum GraphicsDeviceMessage {
    Draw { x: u32, y: u32, data: Vec<u8> },
}


#[derive(Debug)]
pub struct DeviceDrivers {
    basic: Driver<StandardDeviceMessage>,
    keyboard: Option<Driver<KeyboardDeviceMessage>>,
    graphics: Option<Driver<GraphicsDeviceMessage>>,
}

#[derive(Debug)]
pub struct Device {
    name: Option<String>,
    acpi_address: AcpiDeviceAddress,
    pci_address: Option<PciDeviceAddress>,
    drivers: DeviceDrivers,
}


impl Device {
    pub fn from_acpi(aml_name: &AmlName) -> Option<Self> {
        let acpi_address = AcpiDeviceAddress(aml_name.clone());
        let name = acpi_address
            .display_name();
        let pci_address = acpi_address.pci_address();

        let drivers = DeviceDrivers {
            basic: Driver::new(),
            keyboard: None,
            graphics: None
        };

        Some(Self {
            name,
            acpi_address,
            pci_address,
            drivers,
        })
    }
}
