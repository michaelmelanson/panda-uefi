use alloc::vec::Vec;
use aml::{
    resource::{resource_descriptor_list, Resource},
    NamespaceLevel,
};
use conquer_once::spin::OnceCell;

use spin::RwLock;
use x2apic::ioapic::IrqFlags;
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use crate::{
    acpi,
    irq::{configure_irq, enable_irq, end_of_interrupt},
    task,
    util::async_ring_queue::AsyncRingQueue,
};

const KEYBOARD_VECTOR: u8 = 0x23;

static KEYBOARD_COMMAND_PORT: OnceCell<RwLock<Port<u8>>> = OnceCell::uninit();
static KEYBOARD_STATUS_PORT: OnceCell<RwLock<Port<u8>>> = OnceCell::uninit();

static SCANCODE_QUEUE: OnceCell<AsyncRingQueue<u8>> = OnceCell::uninit();

pub fn init_from_acpi_level(acpi_level: NamespaceLevel) {
    log::info!("Starting PC keyboard driver");

    let mut crs = None;

    for (name, value) in acpi_level.values {
        match name.as_str() {
            "_CRS" => crs = Some(value),
            _ => {}
        }
    }

    let mut _irq = None;
    let mut io_ports = Vec::with_capacity(2);

    if let Some(Ok(crs)) = crs.map(acpi::get) {
        let resources = resource_descriptor_list(&crs);

        for resource in resources.unwrap() {
            log::info!("PC Keyboard resource: {resource:X?}");

            match resource {
                Resource::Irq(descriptor) => _irq = Some(descriptor.irq as u8),
                Resource::IOPort(descriptor) => io_ports.push(descriptor.memory_range.0),
                _ => {}
            }
        }
    } else {
        log::error!("Failed to get _CRS");
    }

    let _irq = _irq.expect("No IRQ found for PC keyboard");
    let command_port = Port::new(io_ports[0]);
    let status_port = Port::new(io_ports[1]);

    // for some reason ACPI says it's on IRQ 2, but it's actually on IRQ 1...
    let irq = 1;

    KEYBOARD_COMMAND_PORT.init_once(|| RwLock::new(command_port));
    KEYBOARD_STATUS_PORT.init_once(|| RwLock::new(status_port));
    SCANCODE_QUEUE.init_once(|| AsyncRingQueue::new(100));

    configure_irq(irq, 0, KEYBOARD_VECTOR, IrqFlags::empty(), keyboard_handler)
        .expect("failed to configure keyboard irq");
    enable_irq(0, irq);

    task::start(keyboard_task());
}

async fn keyboard_task() {
    loop {
        let scancode_queue = SCANCODE_QUEUE
            .get()
            .expect("scancode queue not initialized");

        let scancode = scancode_queue.await;
        log::info!("scancode: {:x}", scancode);
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    if let Ok(command_port) = KEYBOARD_COMMAND_PORT.try_get() {
        let mut command_port = command_port.write();
        let scancode = unsafe { command_port.read() };

        if let Ok(queue) = SCANCODE_QUEUE.try_get() {
            if let Err(_) = queue.push(scancode) {
                log::warn!("Keyboard queue full");
            }
        } else {
            log::error!("Keyboard queue not initialized");
        }
    } else {
        log::error!("Keyboard not initialized");
    }

    end_of_interrupt();
}
