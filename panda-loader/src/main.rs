#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(lang_items)]
#![feature(int_roundings)]
#![feature(iter_collect_into)]
#![feature(alloc_layout_extra)]
extern crate alloc;

mod loader;
mod logging;
mod panic;

use core::{alloc::Layout, pin::Pin};

use alloc::{vec, vec::Vec};
use panda_loader_lib::{
    FrameBuffer, KernelEntryFn, LoaderCarePackage, MemoryDescriptor, MemoryDescriptorType,
    PixelFormat,
};
use uefi::{
    prelude::*,
    proto::{
        console::gop::{self, GraphicsOutput},
        media::{
            file::{File, FileAttribute, FileInfo, FileMode, FileType},
            fs::SimpleFileSystem,
        },
    },
    table::{
        boot::{AllocateType, MemoryType},
        cfg::ACPI2_GUID,
        runtime::ResetType,
        Runtime,
    },
    CStr16,
};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size2MiB, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

const PHYSICAL_MEMORY_VIRTUAL_BASE: VirtAddr = unsafe { VirtAddr::new_unsafe(0x000000000) };

struct BootResult {
    entry_point: VirtAddr,
    loader_care_package: LoaderCarePackage,
    system_table: SystemTable<Runtime>,
}

fn uefi_boot(handle: Handle, system_table: SystemTable<Boot>) -> Result<BootResult, uefi::Error> {
    unsafe {
        uefi::alloc::init(system_table.boot_services());
    }

    logging::init(&system_table).unwrap();

    // Print out UEFI revision number
    {
        let rev = system_table.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());

        log::info!("Booted by UEFI {}.{}!", major, minor);
    }

    let mut frame_allocator = ArenaFrameAllocator::from_uefi(&system_table, 5000)?;
    let load_result = {
        let fs = system_table
            .boot_services()
            .locate_protocol::<SimpleFileSystem>()?;
        let fs = unsafe { &mut *fs.get() };

        let mut volume = fs.open_volume().expect("Could not find volume");
        let mut buf = [0u16; 1024];
        let kernel_file = volume.open(
            CStr16::from_str_with_buf("\\EFI\\kernel.elf", &mut buf).unwrap(),
            FileMode::Read,
            FileAttribute::empty(),
        )?;

        let mut kernel_file = if let FileType::Regular(kernel_file) = kernel_file.into_type()? {
            kernel_file
        } else {
            panic!("Kernel image is not a file");
        };

        log::debug!("Found kernel image");

        let mut kernel_image_info_buf = vec![0; 102];
        let kernel_image_info = kernel_file
            .get_info::<FileInfo>(&mut kernel_image_info_buf)
            .expect("Could not get kernel image file info");

        let mut kernel_image = unsafe {
            let length = kernel_image_info.file_size() as usize;
            let layout = Layout::new::<u8>()
                .repeat_packed(length)
                .unwrap()
                .align_to(Size4KiB::SIZE as usize)
                .unwrap();
            let ptr = alloc::alloc::alloc(layout) as *mut u8;
            let slice = core::slice::from_raw_parts_mut(ptr, length);

            Pin::static_mut(slice)
        };

        let bytes_read = kernel_file
            .read(&mut kernel_image[..])
            .expect("could not read kernel image");
        log::debug!("Read {} bytes of kernel image", bytes_read);

        let kernel_object =
            goblin::Object::parse(&kernel_image[..]).expect("Could not parse kernel image");

        let load_result = loader::load_binary(&*kernel_image, &kernel_object, &mut frame_allocator);

        load_result
    };

    let mut rsdp_address = None;

    for table in system_table.config_table() {
        match table.guid {
            ACPI2_GUID => {
                log::debug!(
                    "Found ACPI 2.0 RSDP table at {address:X}",
                    address = table.address as usize
                );
                rsdp_address = Some(PhysAddr::new(table.address as u64))
            }
            _ => {}
        }
    }

    let frame_buffer = framebuffer_from_uefi(&system_table)?;
    let mmap_size = system_table.boot_services().memory_map_size();
    let mut mmap_buf = Vec::new();
    mmap_buf.resize(mmap_size.map_size * 2, 0);
    let mut memory_map = Vec::<MemoryDescriptor>::with_capacity(mmap_size.map_size);

    log::debug!("Exiting boot services...");

    let (system_table, memory_map_iter) = system_table.exit_boot_services(handle, &mut mmap_buf)?;

    // if you get a memory error here, it's probably because something
    // allocated above is being deallocated below, when the allocator has been dropped.
    uefi::alloc::exit_boot_services();
    logging::exit_boot_services();

    log::debug!("Boot services exited, copying memory map...");
    memory_map_iter
        .map(|descriptor| MemoryDescriptor {
            base_addr: PhysAddr::new(descriptor.phys_start),
            length: descriptor.page_count * Size4KiB::SIZE,
            memory_type: match descriptor.ty {
                MemoryType::CONVENTIONAL => MemoryDescriptorType::Available,
                MemoryType::ACPI_RECLAIM => MemoryDescriptorType::AcpiReclaimable,
                _ => MemoryDescriptorType::Reserved,
            },
        })
        .collect_into(&mut memory_map);
    core::mem::forget(mmap_buf);

    let level_4_table = unsafe {
        let (cr3, _) = Cr3::read();
        &mut *(cr3.start_address().as_u64() as *mut PageTable)
    };

    let mut mapper = unsafe { OffsetPageTable::new(level_4_table, PHYSICAL_MEMORY_VIRTUAL_BASE) };

    for mapping in &load_result.mappings {
        let mut flags = PageTableFlags::PRESENT;

        if mapping.writable {
            flags |= PageTableFlags::WRITABLE;
        }

        // if !mapping.executable {
        //     flags |= PageTableFlags::NO_EXECUTE;
        // }

        unsafe {
            let page = Page::<Size2MiB>::from_start_address(mapping.virt_addr).unwrap();

            mapper.unmap(page).expect("Could not unmap page").1.flush();
            mapper
                .map_to(page, mapping.phys_frame, flags, &mut frame_allocator)
                .unwrap()
                .flush();

            log::debug!("Mapped {page:?} to {phys:?}", phys = mapping.phys_frame);
        }
    }

    let entry_point = load_result.entry_point.clone();
    core::mem::forget(load_result);

    let loader_care_package = LoaderCarePackage::new(
        frame_buffer,
        memory_map,
        PHYSICAL_MEMORY_VIRTUAL_BASE,
        rsdp_address,
    );

    Ok(BootResult {
        entry_point,
        loader_care_package,
        system_table,
    })
}

fn framebuffer_from_uefi(system_table: &SystemTable<Boot>) -> Result<FrameBuffer, uefi::Error> {
    let gop = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()?;
    let gop = unsafe { &mut *gop.get() };
    let current_mode_info = gop.current_mode_info().clone();
    let mut frame_buffer = gop.frame_buffer();

    Ok(FrameBuffer {
        base_addr: frame_buffer.as_mut_ptr() as usize,
        resolution: current_mode_info.resolution(),
        stride: current_mode_info.stride(),
        pixel_format: match current_mode_info.pixel_format() {
            gop::PixelFormat::Rgb => PixelFormat::RGB,
            gop::PixelFormat::Bgr => PixelFormat::BGR,
            gop::PixelFormat::Bitmask => todo!(),
            gop::PixelFormat::BltOnly => todo!(),
        },
    })
}

#[entry]
fn uefi_start(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    match uefi_boot(handle, system_table) {
        Ok(BootResult {
            entry_point,
            ref mut loader_care_package,
            system_table,
        }) => {
            let start = unsafe { core::mem::transmute::<u64, KernelEntryFn>(entry_point.as_u64()) };

            log::info!("Starting kernel...");
            start(loader_care_package);
            log::warn!("Kernel returned, shutting down machine");
            unsafe {
                system_table
                    .runtime_services()
                    .reset(ResetType::Shutdown, Status::SUCCESS, None);
            }
        }
        Err(error) => {
            println!("UEFI boot failed: {:?}", error);
            error.status()
        }
    }
}

struct ArenaFrameAllocator {
    end_addr: PhysAddr,
    next_addr: PhysAddr,
}

impl ArenaFrameAllocator {
    fn new(start_addr: PhysAddr, end_addr: PhysAddr) -> Self {
        Self {
            end_addr,
            next_addr: start_addr,
        }
    }

    fn from_uefi(system_table: &SystemTable<Boot>, pages: usize) -> Result<Self, uefi::Error> {
        let start_addr = system_table.boot_services().allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            pages,
        )?;

        let start_addr = PhysAddr::new(start_addr);
        let end_addr = start_addr + (pages * 4096);

        Ok(Self::new(start_addr, end_addr))
    }
}

unsafe impl<S: PageSize> FrameAllocator<S> for ArenaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        if self.next_addr >= self.end_addr {
            log::error!("ArenaFrameAllocator exhausted");
            None
        } else {
            let frame = PhysFrame::containing_address(self.next_addr);
            self.next_addr += S::SIZE;
            Some(frame)
        }
    }
}
