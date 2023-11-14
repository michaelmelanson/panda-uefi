mod ahci_command;
mod ahci_command_table;
pub mod ahci_physical_region_descriptor;
pub mod commands;
mod fis;
mod port;
pub mod register;

mod ide;

use crate::{
    devices::drivers::ahci_controller::{
        ahci_port::{
            ahci_command::AhciCommandHeader,
            ahci_physical_region_descriptor::AhciPhysicalRegionDescriptor,
            commands::identify::IdentifyCommandReply, fis::HostToDeviceRegisterFis,
            ide::ide_identify::IdeIdentifyData,
        },
        ahci_wait_for_port_interrupt,
        registers::{
            AhciCommandAndStatusRegister, AhciPortSataStatusRegister, AhciPortTaskFileDataRegister,
        },
    },
    memory,
};

use self::{
    ahci_command_table::AhciCommandTable,
    commands::read_connected_status::ReadConnectedStatusReply, fis::AhciFis,
    register::AhciPortRegister,
};

use alloc::{boxed::Box, vec};
use ata::ATACommand;
pub use commands::AhciPortCommand;
use futures_util::{
    future::{select, Either},
    pin_mut,
};
pub use port::AhciPort;
use thingbuf::mpsc::Receiver;
use x86_64::VirtAddr;

pub type AhciCommandListStructure = [AhciCommandHeader<[u32; 8]>; 32];

pub async fn ahci_port_task(mut port: AhciPort, channel: Receiver<AhciPortCommand>) {
    // Allocate physical memory for its command list, the received FIS, and its command tables.
    // Make sure the command tables are 128 byte aligned.
    let command_list_stucture: AhciCommandListStructure = Default::default();
    let mut command_list_structure = Box::new(command_list_stucture);

    let mut command_tables: Box<[AhciCommandTable; 32]> = Box::new(Default::default());
    let received_fis = Box::new(AhciFis::default());

    // Memory map these slots as uncacheable.
    unsafe {
        memory::mark_deref_as_uncacheable(command_list_structure.as_ptr());
        memory::mark_deref_as_uncacheable(command_tables.as_ptr());
        memory::mark_deref_as_uncacheable(&*received_fis as *const AhciFis);
    };

    // Set command list and received FIS address registers (and upper registers, if supported).
    let command_list_structure_addr = command_list_structure.as_ptr() as usize as u64;
    let command_list_structure_addr =
        memory::virtual_to_physical(VirtAddr::new(command_list_structure_addr))
            .expect("CLB base address not mapped")
            .as_u64();
    log::info!("Command list structure address: {command_list_structure_addr:016X?}");

    assert_eq!(
        command_list_structure_addr & 0b1111111111,
        0,
        "CLB must be 1K aligned"
    );

    port.write(
        AhciPortRegister::CommandListBaseAddress,
        (command_list_structure_addr >> 0) as u32,
    );
    port.write(
        AhciPortRegister::CommandListBaseAddressUpper,
        (command_list_structure_addr >> 32) as u32,
    );

    let received_fis_addr = &*received_fis as *const AhciFis as usize as u64;
    let received_fis_addr = memory::virtual_to_physical(VirtAddr::new(received_fis_addr))
        .expect("Received FIS base address not mapped")
        .as_u64();
    assert_eq!(
        received_fis_addr & 0b11111111,
        0,
        "FIS base address must be 256-byte aligned"
    );

    log::info!(
        "Port {} Received FIS address: {received_fis_addr:X}",
        port.index()
    );

    port.write(
        AhciPortRegister::FISBaseAddress,
        (received_fis_addr >> 0) as u32,
    );
    port.write(
        AhciPortRegister::FISBaseAddressUpper,
        (received_fis_addr >> 32) as u32,
    );

    let mut pxcmd = AhciCommandAndStatusRegister(port.read(AhciPortRegister::CommandAndStatus));
    pxcmd.set_fis_receive_enable(true);
    port.write(AhciPortRegister::CommandAndStatus, pxcmd.0);

    log::info!(" -> Configured command tables");

    // Reset the port.
    port.reset();
    log::info!(" -> Reset port");

    // Enable interrupts for the port. The D2H bit will signal completed commands.
    port.enable_interrupts();

    // Start command list processing with the port's command register.
    port.start_processing();
    log::info!(" -> Command processing started");

    log::info!("Waiting for commands...");

    while let Some(command) = channel.recv().await {
        match command {
            AhciPortCommand::Noop => log::info!("No-op command"),
            AhciPortCommand::ReadConnectedStatus(reply) => {
                let status = AhciPortSataStatusRegister(port.read(AhciPortRegister::SATAStatus));

                let connected_status = match status.device_detection() {
                    0x0 => ReadConnectedStatusReply::NotPresent,
                    0x1 => ReadConnectedStatusReply::Disconnected,
                    0x3 => ReadConnectedStatusReply::Connected {
                        interface_speed: status.current_interface_speed(),
                        power_state: status.interface_power_mgmt(),
                    },
                    0x4 => ReadConnectedStatusReply::OfflineMode,
                    x => {
                        log::error!("Unknown ATA status: {x:X}");
                        ReadConnectedStatusReply::Unknown
                    }
                };

                log::debug!("Port connected status: {connected_status:?}");
                reply
                    .send(connected_status)
                    .await
                    .expect("Failed to send reply");
            }

            AhciPortCommand::Identify(reply) => {
                log::info!("Received IDENTIFY command");
                let ata_command = ATACommand::Identify;

                let result_data = [0u8; 512];
                let result_data_addr =
                    memory::virtual_to_physical(VirtAddr::new(result_data.as_ptr() as u64))
                        .expect("could not translate result data to physical")
                        .as_u64();

                let mut phys_region = AhciPhysicalRegionDescriptor([0u32; 4]);
                phys_region.set_data_base_addr_upper((result_data_addr >> 32) as u32);
                phys_region.set_data_base_addr_lower(result_data_addr as u32);
                phys_region.set_data_byte_count((result_data.len() - 1) as u32);
                phys_region.set_interrupt_on_completion(true);

                perform_ata_command(
                    &mut port,
                    &mut command_tables,
                    &mut command_list_structure,
                    ata_command,
                    &[phys_region],
                )
                .await;

                let ide_identify =
                    unsafe { core::mem::transmute::<[u8; 512], IdeIdentifyData>(result_data) };
                log::info!("IDE identify result: {ide_identify:?}");
                assert_eq!(ide_identify.signature, 0x0040);

                reply
                    .send(IdentifyCommandReply { ide_identify })
                    .await
                    .expect("Failed to send reply")
            }
            AhciPortCommand::Read(command, reply) => todo!("read command"),
        }
    }
}

fn find_free_slot(port: &AhciPort, command_list: &[AhciCommandTable; 32]) -> Option<usize> {
    let ci = port.read(AhciPortRegister::CommandIssue);
    let sact = port.read(AhciPortRegister::SATAActive);

    for (index, _command_header) in command_list.iter().enumerate() {
        if ci & (1 << index) == 1 {
            // skip; slot has an issued command
            continue;
        }

        if sact & (1 << index) == 1 {
            // skip; slot is active
            continue;
        }

        log::info!("slot {index} is free");
        return Some(index);
    }

    None
}

async fn perform_ata_command(
    port: &mut AhciPort,
    command_tables: &mut [AhciCommandTable; 32],
    command_list_structure: &mut AhciCommandListStructure,
    ata_command: ATACommand,
    phys_regions: &[AhciPhysicalRegionDescriptor<[u32; 4]>],
) {
    let Some(free_slot) = find_free_slot(&port, &command_tables) else {
        todo!("no free slots");
    };
    log::info!("Free slot: {free_slot}");

    log::info!(" -> Configured command list entries");
    let command_table = &mut command_tables[free_slot];
    let command_table_addr = command_table as *mut AhciCommandTable as u64;
    let command_table_addr = memory::virtual_to_physical(VirtAddr::new(command_table_addr))
        .expect("could not translate command tale address to physical")
        .as_u64();
    log::info!(" -> Command table address: {command_table_addr:X}");

    let command_header = &mut command_list_structure[free_slot];
    command_header
        .set_command_table_descriptor_base_address_upper((command_table_addr >> 32) as u32);
    command_header.set_command_table_descriptor_base_address_lower(command_table_addr as u32);

    let mut fis: HostToDeviceRegisterFis<[u8; 64]> =
        HostToDeviceRegisterFis::new(command_table.command_fis);
    fis.set_command(ata_command.into());
    fis.set_command_or_control(true);
    fis.set_device(0);
    fis.set_pmport(0);
    command_table.command_fis = fis.0;
    command_header.set_command_fis_length(1);

    assert!(phys_regions.len() <= 10);
    command_table.phys_region_descriptors[0..phys_regions.len()].copy_from_slice(phys_regions);
    command_header.set_phys_region_descriptor_table_length(phys_regions.len() as u16);

    command_header.set_write(false);
    command_header.set_prefetchable(true);

    // clear any prior errors or pending interrupts
    port.write(AhciPortRegister::SATAError, 0);
    port.write(AhciPortRegister::InterruptStatus, 0);

    log::info!("Sending command to device...");
    port.write(AhciPortRegister::CommandIssue, 1 << free_slot);

    log::info!("Waiting for completion");
    ahci_wait_for_port_interrupt(port.index()).await;

    // the command should have completed by now.
    let pxci = port.read(AhciPortRegister::CommandIssue);
    loop {
        if pxci & (1 << free_slot) == 0 {
            break;
        }
    }

    log::info!(
        "Command complete, error register is {:X}",
        port.read(AhciPortRegister::SATAError)
    );
}
