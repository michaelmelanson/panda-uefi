mod frame_allocator;

#[cfg(not(test))]
use core::alloc::Layout;
use core::mem::size_of;

use linked_list_allocator::LockedHeap;
use panda_loader_lib::MemoryDescriptor;
use x86_64::{
    structures::paging::{
        mapper::{MapToError, TranslateResult, UnmapError},
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size2MiB, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use self::frame_allocator::PhysicalAllocator;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

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

pub fn physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    unsafe { VirtAddr::new(physical_address.as_u64() + PHYSICAL_MEMORY_VIRTUAL_BASE.as_u64()) }
}

pub unsafe fn physical_memory_ref<T>(physical_address: PhysAddr) -> &'static mut T {
    let virtual_address = physical_to_virtual(physical_address);
    &mut *virtual_address.as_mut_ptr()
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

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

pub(crate) unsafe fn mark_deref_as_uncacheable<T>(ptr: *const T) {
    let start_address = VirtAddr::from_ptr(ptr);
    mark_as_uncacheable(start_address, start_address + size_of::<T>())
}

pub(crate) unsafe fn mark_as_uncacheable(start_address: VirtAddr, end_address: VirtAddr) {
    let mut mapper = page_mapper();

    let range = Page::<Size2MiB>::range(
        Page::containing_address(start_address),
        Page::containing_address(end_address),
    );

    for page in range {
        match mapper.translate(page.start_address()) {
            TranslateResult::Mapped {
                frame: _,
                offset: _,
                mut flags,
            } => {
                flags.set(PageTableFlags::NO_CACHE, true);

                mapper
                    .update_flags(page, flags)
                    .expect("Failed to update memory page flags")
                    .flush();
            }

            TranslateResult::NotMapped => todo!(),
            TranslateResult::InvalidFrameAddress(_) => todo!(),
        }
    }
}

pub fn virtual_to_physical(addr: VirtAddr) -> Option<PhysAddr> {
    let mut mapper = unsafe { page_mapper() };
    mapper.translate_addr(addr)
}
