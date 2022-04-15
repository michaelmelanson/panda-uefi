use x86_64::PhysAddr;

#[derive(Debug)]
pub struct MemoryDescriptor {
    pub base_addr: PhysAddr,
    pub length: u64,
    pub memory_type: MemoryDescriptorType,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MemoryDescriptorType {
    // the memory is available for use by the kernel
    Available,

    // the memory should not be used by the kernel
    Reserved,

    // the memory may be used once data has been read from the ACPI tables
    AcpiReclaimable,
}
