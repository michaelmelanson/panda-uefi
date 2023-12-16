use acpi::PciConfigRegions;
use core::mem;
use linked_list_allocator::LockedHeap;
use spin::once::Once;
use x86_64::PhysAddr;

use crate::memory;

static PCI_CONFIG_REGIONS: Once<PciConfigRegions<'static, &'static LockedHeap>> = Once::new();

pub fn init(pci_config_regions: PciConfigRegions<'static, &'static LockedHeap>) {
    PCI_CONFIG_REGIONS.call_once(move || pci_config_regions);

    log::info!("PCI initialized");
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PciRegister {
    VendorId,
    DeviceId,
    Command,
    Status,
    RevisionId,
    ProgIf,
    ClassCode,
    SubclassCode,
    CacheLineSize,
    LatencyTimer,
    HeaderType,
    Bist,
    BaseAddress0,
    BaseAddress1,
    BaseAddress2,
    BaseAddress3,
    BaseAddress4,
    BaseAddress5,
    CardbusCisPtr,
    SubsystemVendorId,
    SubsystemId,
    ExpansionRomBaseAddress,
    CapabilitiesPtr,
    Reserved1,
    Reserved2,
    InterruptLine,
    InterruptPin,
    MinGrant,
    MaxLatency,
}

impl PciRegister {
    pub fn offset(&self) -> u16 {
        match self {
            PciRegister::VendorId => 0x00,
            PciRegister::DeviceId => 0x02,

            PciRegister::Command => 0x04,
            PciRegister::Status => 0x06,

            PciRegister::RevisionId => 0x08,
            PciRegister::ProgIf => 0x09,
            PciRegister::SubclassCode => 0x0a,
            PciRegister::ClassCode => 0x0b,

            PciRegister::CacheLineSize => 0x0c,
            PciRegister::LatencyTimer => 0x0d,
            PciRegister::HeaderType => 0x0e,
            PciRegister::Bist => 0x0f,

            PciRegister::BaseAddress0 => 0x10,
            PciRegister::BaseAddress1 => 0x14,
            PciRegister::BaseAddress2 => 0x18,
            PciRegister::BaseAddress3 => 0x1c,
            PciRegister::BaseAddress4 => 0x20,
            PciRegister::BaseAddress5 => 0x24,
            PciRegister::CardbusCisPtr => 0x28,
            PciRegister::SubsystemVendorId => 0x2c,
            PciRegister::SubsystemId => 0x2e,
            PciRegister::ExpansionRomBaseAddress => 0x30,
            PciRegister::CapabilitiesPtr => 0x34,
            PciRegister::Reserved1 => 0x38,
            PciRegister::Reserved2 => 0x3c,
            PciRegister::InterruptLine => 0x3c,
            PciRegister::InterruptPin => 0x3d,
            PciRegister::MinGrant => 0x3e,
            PciRegister::MaxLatency => 0x3f,
        }
    }

    const fn width(&self) -> u8 {
        match self {
            PciRegister::VendorId
            | PciRegister::DeviceId
            | PciRegister::Command
            | PciRegister::Status => 2,
            PciRegister::RevisionId
            | PciRegister::ClassCode
            | PciRegister::SubclassCode
            | PciRegister::ProgIf
            | PciRegister::CacheLineSize
            | PciRegister::LatencyTimer
            | PciRegister::HeaderType
            | PciRegister::Bist => 1,
            PciRegister::BaseAddress0
            | PciRegister::BaseAddress1
            | PciRegister::BaseAddress2
            | PciRegister::BaseAddress3
            | PciRegister::BaseAddress4
            | PciRegister::BaseAddress5
            | PciRegister::CardbusCisPtr => 4,
            PciRegister::SubsystemVendorId | PciRegister::SubsystemId => 2,
            PciRegister::ExpansionRomBaseAddress => 4,
            PciRegister::CapabilitiesPtr
            | PciRegister::Reserved1
            | PciRegister::Reserved2
            | PciRegister::InterruptLine
            | PciRegister::InterruptPin
            | PciRegister::MinGrant
            | PciRegister::MaxLatency => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciDevice {
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        PciDevice {
            segment,
            bus,
            device,
            function,
        }
    }

    pub fn read<T: 'static + Copy>(&self, register: PciRegister) -> T {
        if mem::size_of::<T>() != register.width() as usize {
            panic!("Invalid register width for {:?}", register);
        }

        let result = read(
            self.segment,
            self.bus,
            self.device,
            self.function,
            register.offset(),
        );

        match result {
            Ok(value) => value,
            Err(error) => panic!("PCI read failed: {:?}", error),
        }
    }

    pub fn write<T: 'static + Copy>(&self, register: PciRegister, value: T) {
        if mem::size_of::<T>() != register.width() as usize {
            panic!("Invalid register width for {:?}", register);
        }

        let result = write(
            self.segment,
            self.bus,
            self.device,
            self.function,
            register.offset(),
            value,
        );

        match result {
            Ok(()) => (),
            Err(error) => panic!("PCI write failed: {:?}", error),
        }
    }
}

#[derive(Debug)]
pub enum PciError {
    NotInitialized,
    InvalidPciAddress,
}

pub(crate) fn read<T: 'static + Copy>(
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
    offset: u16,
) -> Result<T, PciError> {
    let pci_config_regions = PCI_CONFIG_REGIONS.get().ok_or(PciError::NotInitialized)?;

    let base_addr = pci_config_regions
        .physical_address(segment, bus, device, function)
        .ok_or(PciError::InvalidPciAddress)?;
    let addr = memory::physical_to_virtual(PhysAddr::new(base_addr + offset as u64));

    let value = unsafe { addr.as_ptr::<T>().read_volatile() };
    Ok(value)
}

pub(crate) fn write<T: 'static + Copy>(
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
    offset: u16,
    value: T,
) -> Result<(), PciError> {
    let pci_config_regions = PCI_CONFIG_REGIONS.get().ok_or(PciError::NotInitialized)?;

    let base_addr = pci_config_regions
        .physical_address(segment, bus, device, function)
        .ok_or(PciError::InvalidPciAddress)?;
    let addr = memory::physical_to_virtual(PhysAddr::new(base_addr + offset as u64));

    unsafe {
        addr.as_mut_ptr::<T>().write_volatile(value);
    }
    Ok(())
}
