use alloc::vec::Vec;
use goblin::elf::Elf;

use x86_64::{
    structures::paging::{FrameAllocator, PageSize},
    VirtAddr,
};

use crate::loader::{FrameMapping, LoadResult};

pub fn load_elf_binary<S: PageSize, A: FrameAllocator<S>>(
    file: &[u8],
    binary: &Elf,
    frame_allocator: &mut A,
) -> LoadResult<S> {
    log::info!("Entry point at {:#X}", binary.entry);

    let mut mappings = Vec::new();

    for section in binary.section_headers.iter() {
        let name = binary.shdr_strtab.get_at(section.sh_name).unwrap();
        log::debug!(
            "Section {name}: addr={addr:#08X}, size={size}",
            addr = section.sh_addr,
            size = section.sh_size
        );

        if section.is_alloc() {
            if section.sh_addralign != S::SIZE {
                panic!(
                    "Section {name} has {actual}-byte alignment, expected {expected}",
                    name = name,
                    actual = section.sh_addralign,
                    expected = S::SIZE
                );
            }

            let pages_needed = section.sh_size.div_ceil(S::SIZE);
            for i in 0..pages_needed {
                let offset = i * S::SIZE;

                unsafe {
                    // allocate page-aligned memory
                    let phys_frame = frame_allocator.allocate_frame().unwrap();

                    let page_slice = core::slice::from_raw_parts_mut(
                        phys_frame.start_address().as_u64() as *mut u8,
                        S::SIZE as usize,
                    );

                    // copy over data from file
                    let bytes = (section.sh_size - offset).min(S::SIZE);
                    page_slice[0..bytes as usize].copy_from_slice(
                        &file[(section.sh_offset + offset) as usize
                            ..(section.sh_offset + offset + bytes) as usize],
                    );

                    let virt_addr = VirtAddr::new(section.sh_addr + offset);

                    mappings.push(FrameMapping {
                        phys_frame,
                        virt_addr,
                        writable: section.is_writable(),
                        executable: section.is_executable(),
                    });
                }
            }
        }
    }

    LoadResult {
        entry_point: VirtAddr::new(binary.entry),
        mappings,
    }
}
