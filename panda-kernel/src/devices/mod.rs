use alloc::{string::String, sync::Arc, vec::Vec};
use aml::AmlName;
use crossbeam_queue::SegQueue;

use self::address::{acpi_device_address::AcpiDeviceAddress, pci_device_address::PciDeviceAddress};

pub mod address;
pub mod drivers;

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

pub enum KeyboardDeviceMessage {}

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
        let acpi_address = AcpiDeviceAddress::new(aml_name.clone());
        let name = acpi_address.display_name();
        let pci_address = acpi_address.pci_address();

        let drivers = DeviceDrivers {
            basic: Driver::new(),
            keyboard: None,
            graphics: None,
        };

        Some(Self {
            name,
            acpi_address,
            pci_address,
            drivers,
        })
    }
}
