#![no_std]
extern crate alloc;

mod frame_buffer;
mod memory_descriptor;

use alloc::vec::Vec;

pub use frame_buffer::{FrameBuffer, PixelFormat};
pub use memory_descriptor::{MemoryDescriptor, MemoryDescriptorType};
use x86_64::{PhysAddr, VirtAddr};

pub type KernelEntryFn = extern "win64" fn(&LoaderCarePackage);

const CARE_PACKAGE_MAGIC_NUMBER: u64 = 0x5542_5542_5542_5542;

#[derive(Debug)]
pub enum LoaderCarePackageError {
    InvalidMagicNumber,
}

#[derive(Debug)]
pub struct LoaderCarePackage {
    magic_number: u64,
    pub frame_buffer: FrameBuffer,
    pub memory_map: Vec<MemoryDescriptor>,
    pub phys_memory_virt_offset: VirtAddr,
    pub rsdp_address: Option<PhysAddr>,
}

impl LoaderCarePackage {
    pub fn new(
        frame_buffer: FrameBuffer,
        memory_map: Vec<MemoryDescriptor>,
        phys_memory_virt_offset: VirtAddr,
        rsdp_address: Option<PhysAddr>,
    ) -> Self {
        LoaderCarePackage {
            magic_number: CARE_PACKAGE_MAGIC_NUMBER,
            frame_buffer,
            memory_map,
            phys_memory_virt_offset,
            rsdp_address,
        }
    }

    pub fn validate(&self) -> Result<(), LoaderCarePackageError> {
        if self.magic_number != CARE_PACKAGE_MAGIC_NUMBER {
            return Err(LoaderCarePackageError::InvalidMagicNumber);
        }

        Ok(())
    }
}
