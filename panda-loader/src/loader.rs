mod elf;
use alloc::vec::Vec;
use x86_64::{
    structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub struct FrameMapping<S: PageSize> {
    pub phys_frame: PhysFrame<S>,
    pub virt_addr: VirtAddr,
    pub writable: bool,
    pub executable: bool,
}

pub struct LoadResult<S: PageSize> {
    pub entry_point: VirtAddr,
    pub mappings: Vec<FrameMapping<S>>,
}

pub fn load_binary<S: PageSize>(
    file: &[u8],
    object: &goblin::Object,
    base_addr: PhysAddr,
    frame_allocator: &mut impl FrameAllocator<S>,
) -> LoadResult<S> {
    if !base_addr.is_aligned(Size4KiB::SIZE) {
        panic!("Base address must be page aligned");
    }

    match object {
        goblin::Object::Elf(binary) => {
            elf::load_elf_binary(file, binary, base_addr, frame_allocator)
        }
        goblin::Object::PE(_) => unimplemented!("PE kernel"),
        goblin::Object::Mach(_) => unimplemented!("Mach kernel"),
        goblin::Object::Archive(_) => unimplemented!("Archive kernel"),
        goblin::Object::Unknown(_) => unimplemented!("Unknown kernel"),
    }
}
