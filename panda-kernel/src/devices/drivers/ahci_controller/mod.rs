use alloc::vec::Vec;
use bitfield::Bit;
use conquer_once::spin::OnceCell;
use thingbuf::mpsc::channel;
use x2apic::ioapic::IrqFlags;
use x86_64::structures::idt::InterruptStackFrame;

use crate::{
    devices::drivers::ahci_controller::{
        ahci_controller::AhciController,
        ahci_port::{
            ahci_port_task,
            commands::{
                read::ReadCommand, read_connected_status::ReadConnectedStatusReply, AhciPortCommand,
            },
            register::AhciPortRegister,
        },
        ahci_register::AhciRegister,
        registers::AhciPortInterruptStatusRegister,
    },
    irq::{configure_irq, enable_irq, end_of_interrupt},
    memory,
    pci::{PciDevice, PciRegister},
    task,
    util::async_ring_queue::AsyncRingQueue,
};

use self::registers::AhciPciCommandRegister;

mod ahci_controller;
pub mod ahci_port;
mod ahci_register;
mod registers;

const AHCI_VECTOR: u8 = 0x24;

pub fn init_from_pci_device(pci_device: PciDevice) {
    let controller = AhciController::new(pci_device);

    log::info!(
        "Starting AHCI controller at {ahci_base:#X}",
        ahci_base = controller.ahci_base_addr()
    );

    AHCI_CONTROLLER.init_once(|| controller.clone());
    AHCI_PORT_EVENTS.init_once(|| {
        let mut queues = Vec::new();
        queues.reserve(MAX_PORTS as usize);

        for _ in 0..MAX_PORTS {
            queues.push(AsyncRingQueue::new(10));
        }

        queues
    });

    task::start(ahci_controller_task(controller));
}

async fn ahci_controller_task(mut controller: AhciController) {
    log::info!("Starting AHCI controller...");

    // Enable interrupts, DMA, and memory space access in the PCI command register
    let mut command = AhciPciCommandRegister(controller.pci_device().read(PciRegister::Command));
    command.set_interrupt_disable(false);
    command.set_memory_space_enable(true);
    command.set_bus_master_enable(true);
    controller
        .pci_device()
        .write(PciRegister::Command, command.0);
    log::info!("  -> Configured PCI device");

    // Memory map BAR 5 register as uncacheable.
    let bar5 = controller.ahci_base_addr();
    unsafe {
        memory::mark_as_uncacheable(bar5, bar5);
    }

    controller.perform_bios_os_handoff();
    controller.hba_reset();

    // Register IRQ handler, using interrupt line given in the PCI register. This interrupt line may be shared with other devices, so the usual implications of this apply.
    let irq = controller.pci_device().read(PciRegister::InterruptLine);
    configure_irq(irq, 0, AHCI_VECTOR, IrqFlags::empty(), ahci_irq_handler)
        .expect("failed to configure keyboard irq");
    enable_irq(0, irq);

    // Enable AHCI mode and interrupts in global host control register.
    controller.enable_ahci_and_interrupts();

    // Read capabilities registers. Check 64-bit DMA is supported if you need it.
    let capabilities = controller.capabilities();

    log::info!("AHCI controller initialized, configuring ports...");

    let mut ports = Vec::with_capacity(capabilities.number_of_ports() as usize);

    for index in 0..=capabilities.number_of_ports() {
        if let Some(port) = controller.port(index) {
            let (sender, receiver) = channel(10);
            task::start(ahci_port_task(port, receiver));
            ports.push((index, sender));
        }
    }

    log::info!("Checking port status...");
    for (index, port) in ports {
        //     Read signature/status of the port to see if it connected to a drive.
        let (sender, receiver) = channel(1);
        port.send(AhciPortCommand::ReadConnectedStatus(sender))
            .await
            .expect("Failed to send AHCI port command");

        let Some(reply) = receiver.recv().await else {
            continue;
        };

        log::info!(" -> port {index} status: {reply:?}");

        match reply {
            ReadConnectedStatusReply::Connected { .. } => {}
            _ => continue,
        };

        //     Send IDENTIFY ATA command to connected drives. Get their sector size and count.
        let (sender, receiver) = channel(1);
        port.send(AhciPortCommand::Identify(sender))
            .await
            .expect("Failed to send ATA IDENTIFY command");

        let Some(reply) = receiver.recv().await else {
            panic!("failed to IDENTIFY drive");
        };
        log::info!(" -> IDENTIFY response: {reply:?}");

        let identify_data = reply.ide_identify;

        //     Read the master boot record
        let (sender, receiver) = channel(1);
        port.send(AhciPortCommand::Read(
            ReadCommand {
                start_sector: 0,
                sector_count: 1,
            },
            sender,
        ))
        .await
        .expect("Failed to send ATA READ command");

        let Some(reply) = receiver.recv().await else {
            panic!("READ failed");
        };

        log::info!(" -> READ response: {reply:X?}");

        let mbr = reply.data[0];
        let partition_entry_0 = &mbr[0x1BE..0x01CE];
        log::info!("     -> Partition entry 0: {:?}", &partition_entry_0);
        let start_sector_c =
            partition_entry_0[0x03] as u64 + (((partition_entry_0[0x02] >> 6) as u64) << 8);
        let start_sector_h = partition_entry_0[0x01] as u64;
        let start_sector_s = (partition_entry_0[0x03] & 0b111111) as u64;

        log::info!(
            "     -> Start sector: CHS=({start_sector_c}, {start_sector_h}, {start_sector_s})"
        );

        let start_sector = (((start_sector_c * (identify_data.heads as u64)) + start_sector_h)
            * (identify_data.sectors as u64))
            + start_sector_s;

        log::info!("     -> Start sector: LBA={start_sector}");

        let (sender, receiver) = channel(1);
        port.send(AhciPortCommand::Read(
            ReadCommand {
                start_sector,
                sector_count: 1,
            },
            sender,
        ))
        .await
        .expect("Failed to send ATA READ command");

        let Some(reply) = receiver.recv().await else {
            panic!("READ failed");
        };

        log::info!(" -> READ response: {reply:?}");
    }

    log::info!("AHCI controller started");
}

static AHCI_CONTROLLER: OnceCell<AhciController> = OnceCell::uninit();
const MAX_PORTS: u8 = 32;
static AHCI_PORT_EVENTS: OnceCell<Vec<AsyncRingQueue<AhciPortInterruptStatusRegister>>> =
    OnceCell::uninit();

pub async fn ahci_wait_for_port_interrupt(port_index: u8) -> AhciPortInterruptStatusRegister {
    let Some(event_queues) = AHCI_PORT_EVENTS.get() else {
        panic!("AHCI event queues not initialized");
    };

    log::info!("Waiting for event on port {port_index}...");
    let queue = &event_queues[port_index as usize];
    let event = queue.await;
    log::info!("Received event on port {port_index}: {event:?}");

    event
}

extern "x86-interrupt" fn ahci_irq_handler(_stack_frame: InterruptStackFrame) {
    log::info!("AHCI IRQ!");

    if let Some(ahci_controller) = AHCI_CONTROLLER.get() {
        let interrupt_status = ahci_controller.read(AhciRegister::InterruptStatus);

        log::info!("Interrupt status = {:b}", interrupt_status);

        if let Some(event_queues) = AHCI_PORT_EVENTS.get() {
            for i in 0..MAX_PORTS {
                if interrupt_status.bit(i as usize) {
                    let mut port = ahci_controller
                        .port(i)
                        .expect("Interrupt for invalid port {i}");

                    let pxis = AhciPortInterruptStatusRegister(
                        port.read(AhciPortRegister::InterruptStatus),
                    );

                    if let Err(_) = event_queues[i as usize].push(pxis.clone()) {
                        log::info!("Event overflow on AHCI port {i}");
                    }

                    port.write(AhciPortRegister::InterruptStatus, pxis.0);
                }
            }
        }

        ahci_controller.write(AhciRegister::InterruptStatus, interrupt_status);
    }

    // if let Ok(command_port) = KEYBOARD_COMMAND_PORT.try_get() {
    //     let mut command_port = command_port.write();
    //     let scancode = unsafe { command_port.read() };

    //     if let Ok(queue) = SCANCODE_QUEUE.try_get() {
    //         if let Err(_) = queue.push(scancode) {
    //             log::warn!("Keyboard queue full");
    //         }
    //     } else {
    //         log::error!("Keyboard queue not initialized");
    //     }

    // } else {
    //     log::error!("Keyboard not initialized");
    // }

    end_of_interrupt();
}
