use panda_loader_lib::{MemoryDescriptor, MemoryDescriptorType};
use x86_64::{
    structures::paging::{FrameAllocator, PageSize, PhysFrame},
    PhysAddr,
};

const MAX_REGIONS: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub addr_range: (PhysAddr, PhysAddr),
    pub next_addr: PhysAddr,
}

impl From<&MemoryDescriptor> for MemoryRegion {
    fn from(descriptor: &MemoryDescriptor) -> Self {
        MemoryRegion {
            addr_range: (
                descriptor.base_addr,
                descriptor.base_addr + descriptor.length,
            ),
            next_addr: descriptor.base_addr,
        }
    }
}

#[derive(Default)]
pub struct PhysicalAllocator {
    pub memory_regions: [Option<MemoryRegion>; MAX_REGIONS],
    pub current_region: usize,
}

impl PhysicalAllocator {
    pub const fn new() -> Self {
        PhysicalAllocator {
            memory_regions: [None; MAX_REGIONS],
            current_region: 0,
        }
    }

    pub fn init(&mut self, descriptors: &[MemoryDescriptor]) {
        let available_descriptors = descriptors
            .iter()
            .filter(|descriptor| descriptor.memory_type == MemoryDescriptorType::Available);

        for (i, descriptor) in available_descriptors.enumerate() {
            let region = MemoryRegion::from(descriptor);
            self.memory_regions[i] = Some(region);
        }
    }
}

unsafe impl<S: PageSize> FrameAllocator<S> for PhysicalAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        for region in &mut self.memory_regions[self.current_region..] {
            if let Some(region) = region {
                let frame = PhysFrame::from_start_address(region.next_addr).unwrap();
                region.next_addr += frame.size();

                // advance to next region if this one's empty
                if region.next_addr >= region.addr_range.1 {
                    self.current_region += 1;
                }
                return Some(frame);
            }
        }

        None
    }
}
