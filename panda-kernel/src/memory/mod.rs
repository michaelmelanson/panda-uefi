mod frame_allocator;

use core::alloc::Layout;

use linked_list_allocator::LockedHeap;
use panda_loader_lib::MemoryDescriptor;
use x86_64::{
    structures::paging::{
        mapper::{MapToError, UnmapError},
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size2MiB, Size4KiB,
    },
    VirtAddr,
};

use self::frame_allocator::PhysicalAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut FRAME_ALLOCATOR: PhysicalAllocator = PhysicalAllocator::new();

static mut PHYSICAL_MEMORY_VIRTUAL_BASE: VirtAddr = unsafe { VirtAddr::new_unsafe(0x000000000) };

#[derive(Debug)]
pub enum MemoryError {
    UnmapError(UnmapError),
    MapToError4KiB(MapToError<Size4KiB>),
}

impl From<UnmapError> for MemoryError {
    fn from(error: UnmapError) -> Self {
        MemoryError::UnmapError(error)
    }
}

impl From<MapToError<Size4KiB>> for MemoryError {
    fn from(error: MapToError<Size4KiB>) -> Self {
        MemoryError::MapToError4KiB(error)
    }
}

pub unsafe fn page_table() -> &'static mut PageTable {
    // read from CR3 register
    let (cr3, _flags) = x86_64::registers::control::Cr3::read();

    // convert to a physical address
    let phys_addr = cr3.start_address();

    // convert to a virtual address
    let virt_addr = VirtAddr::new(phys_addr.as_u64() + PHYSICAL_MEMORY_VIRTUAL_BASE.as_u64());

    &mut *virt_addr.as_mut_ptr()
}

pub unsafe fn page_mapper() -> OffsetPageTable<'static> {
    OffsetPageTable::new(page_table(), PHYSICAL_MEMORY_VIRTUAL_BASE)
}

pub fn allocate_frame() -> Option<PhysFrame> {
    unsafe { FRAME_ALLOCATOR.allocate_frame() }
}

pub fn map_page_to_frame(page: Page, frame: PhysFrame) -> Result<(), MemoryError> {
    unsafe {
        let mut mapper = page_mapper();
        mapper
            .map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut FRAME_ALLOCATOR,
            )?
            .flush();
    }

    Ok(())
}

pub unsafe fn init_page_table() -> Result<(), MemoryError> {
    // unmap 0xD0000000 - 0xDFFFFFFF
    let mut mapper = page_mapper();

    for addr in (0xD0000000..0xE0000000).step_by(Size2MiB::SIZE as usize) {
        let page = Page::<Size2MiB>::containing_address(VirtAddr::new(addr));
        let (_, flush) = mapper.unmap(page)?;
        flush.flush();
    }

    Ok(())
}

pub fn init(descriptors: &[MemoryDescriptor], phys_mem_base: VirtAddr) -> Result<(), MemoryError> {
    unsafe {
        PHYSICAL_MEMORY_VIRTUAL_BASE = phys_mem_base;
        FRAME_ALLOCATOR.init(descriptors);

        init_page_table()?;

        x86_64::instructions::interrupts::enable();

        let heap_range = 0xD0000000..0xE0000000;
        GLOBAL_ALLOCATOR
            .lock()
            .init(heap_range.start, heap_range.count());
    }

    Ok(())
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
