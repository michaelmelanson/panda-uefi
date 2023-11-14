use super::ahci_physical_region_descriptor::AhciPhysicalRegionDescriptor;

const NUM_PHYS_REGION_DESCRIPTORS: usize = 10;
const PHYS_REGION_DESCRIPTOR_ITEM_SIZE: usize = 16;

#[repr(C, align(128))]
pub struct AhciCommandTable {
    pub command_fis: [u8; 0x40],
    pub ahci_command: [u8; 0x10],
    reserved: [u8; 0x30],

    pub phys_region_descriptors:
        [AhciPhysicalRegionDescriptor<[u32; 4]>; NUM_PHYS_REGION_DESCRIPTORS],
}

impl Default for AhciCommandTable {
    fn default() -> Self {
        Self {
            command_fis: [0; 0x40],
            ahci_command: [0; 0x10],
            reserved: [0; 0x30],
            phys_region_descriptors: [AhciPhysicalRegionDescriptor([0u32; 4]);
                NUM_PHYS_REGION_DESCRIPTORS],
        }
    }
}
