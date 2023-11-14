use crate::devices::drivers::ahci_controller::{
    ahci_controller::AhciController,
    registers::{
        AhciCommandAndStatusRegister, AhciPortInterruptEnableRegister, AhciPortSataControlRegister,
        DeviceDetectionInit,
    },
};
use x86_64::VirtAddr;

use super::register::AhciPortRegister;

pub struct AhciPort {
    addr: VirtAddr,
    index: u8,
}

impl AhciPort {
    /// Unsafe because the caller should guarantee that no other references to
    /// this port exist.
    pub fn new(controller: &AhciController, index: u8) -> Self {
        if index >= 32 {
            panic!("Invalid AHCI port number {}", index);
        }

        let addr = controller.ahci_base_addr() + (0x100 + (0x80 * index as usize));
        log::info!("AHCI port {index} at {addr:X}", addr = addr.as_u64());
        Self { addr, index }
    }

    pub fn read(&self, register: AhciPortRegister) -> u32 {
        let addr = self.addr + register.offset();
        let value = unsafe { addr.as_ptr::<u32>().read_volatile() };
        value
    }

    pub fn write(&mut self, register: AhciPortRegister, value: u32) {
        let addr = self.addr + register.offset();
        unsafe { addr.as_mut_ptr::<u32>().write_volatile(value) }
    }

    pub(crate) fn stop_processing(&mut self) {
        let mut pxcmd = AhciCommandAndStatusRegister(self.read(AhciPortRegister::CommandAndStatus));
        pxcmd.set_start(false);
        self.write(AhciPortRegister::CommandAndStatus, pxcmd.0);

        loop {
            pxcmd = AhciCommandAndStatusRegister(self.read(AhciPortRegister::CommandAndStatus));
            if !pxcmd.command_list_running() {
                break;
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        log::info!("Resetting port...");

        self.stop_processing();
        log::info!("  -> Processing stopped");

        let mut pxsctl = AhciPortSataControlRegister(self.read(AhciPortRegister::SATAControl));
        pxsctl.set_device_detection_init(DeviceDetectionInit::InitCommunication as u8);
        self.write(AhciPortRegister::SATAControl, pxsctl.0);
        log::info!("  -> Device detection init set");

        // wait at least 1ms
        wait();

        log::info!("  -> Device detection init");
        pxsctl = AhciPortSataControlRegister(self.read(AhciPortRegister::SATAControl));
        pxsctl.set_device_detection_init(DeviceDetectionInit::NoOp as u8);
        self.write(AhciPortRegister::SATAControl, pxsctl.0);
        log::info!("  -> Device detection init cleared, waiting for device to reset");

        // loop {
        //     let pxssts = AhciPortSataStatusRegister(self.read(AhciPortRegister::SATAStatus));
        //     if pxssts.device_detection() == 0x3 {
        //         break;
        //     }

        //     // wait at least 1ms
        //     let start = unsafe { rdtsc() };
        //     loop {
        //         let now = unsafe { rdtsc() };

        //         if now > start + 10_000_000_000 {
        //             break;
        //         }
        //     }
        // }
        // log::info!("  -> Device reset");

        self.write(AhciPortRegister::SATAError, 0xffffffff);
        log::info!("  -> Errors cleared");
        log::info!("All done resetting");
    }

    pub(crate) fn start_processing(&mut self) {
        log::info!("Starting command processing...");

        let mut pxcmd = AhciCommandAndStatusRegister(self.read(AhciPortRegister::CommandAndStatus));
        assert_eq!(pxcmd.start(), false);
        assert_eq!(pxcmd.command_list_running(), false);
        assert_eq!(pxcmd.fis_receive_enable(), true);

        pxcmd.set_start(true);
        pxcmd.set_power_on_device(true);
        pxcmd.set_spin_up_device(true);
        self.write(AhciPortRegister::CommandAndStatus, pxcmd.0);

        loop {
            pxcmd = AhciCommandAndStatusRegister(self.read(AhciPortRegister::CommandAndStatus));
            if pxcmd.command_list_running() {
                break;
            }

            wait();
        }
        log::info!("Command processing started");
    }

    pub fn enable_interrupts(&mut self) {
        let mut pxie =
            AhciPortInterruptEnableRegister(self.read(AhciPortRegister::InterruptEnable));
        pxie.set_device_to_host_register_fis_interrupt_enable(true);
        pxie.set_interface_fatal_error_enable(true);
        pxie.set_interface_non_fatal_error_enable(true);
        pxie.set_host_bus_data_error_enable(true);
        pxie.set_host_bus_fatal_error_enable(true);
        pxie.set_descriptor_processed_interupt_enable(true);
        pxie.set_task_file_error_enable(true);
        self.write(AhciPortRegister::InterruptEnable, pxie.0);
    }

    pub fn index(&self) -> u8 {
        self.index
    }
}

extern "C" {
    #[link_name = "llvm.x86.rdtsc"]
    fn rdtsc() -> u64;
}

pub fn wait() {
    let start = unsafe { rdtsc() };
    loop {
        let now = unsafe { rdtsc() };

        if now > start + 100_000_000 {
            break;
        }
    }
}
