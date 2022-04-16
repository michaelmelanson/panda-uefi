use core::ptr::NonNull;

use ::acpi::PhysicalMapping;
use x86_64::PhysAddr;

use crate::memory;

#[derive(Clone, Copy)]
pub struct AcpiMemoryHandler;

impl ::acpi::AcpiHandler for AcpiMemoryHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let physical_address = PhysAddr::new(physical_address as u64);
        let virtual_addr = memory::physical_to_virtual(physical_address);

        PhysicalMapping::new(
            physical_address.as_u64() as usize,
            NonNull::new(virtual_addr.as_mut_ptr()).unwrap(),
            size,
            size,
            *self,
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // nothing to do
    }
}
