// Source, page 24-25 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use core::fmt::Debug;

use bitfield::bitfield;

bitfield! {
    pub struct AhciPortInterruptStatusRegister(u32);

    pub u8, cold_port_detect_status, set_cold_port_detect_status: 31;
    pub u8, task_file_error_status, set_task_file_error_status: 30;
    pub u8, host_bus_fatal_error_status, set_host_bus_fatal_error_status: 29;
    pub u8, host_bus_data_error_status, set_host_bus_data_error_status: 28;
    pub u8, interface_fatal_error_status, set_interface_fatal_error_status: 27;
    pub u8, interface_non_fatal_error_status, set_interface_non_fatal_error_status: 26;
    // 25 reserved
    pub u8, overflow_status, set_overflow_status: 24;
    pub u8, incorrect_port_multiplier_status, set_incorrect_port_multiplier_status: 23;
    pub u8, phyrdy_change_status, _: 22; // ro
    // 21:08 reserved
    pub u8, device_mechanical_presence_status, set_device_mechanical_presence_status: 7;
    pub u8, port_connect_change_status, _: 6;
    pub u8, descriptor_processed, set_descriptor_processed: 5;
    pub u8, unknown_fis_interrrupt, _: 4;
    pub u8, set_device_bits_interrupt, set_set_device_bits_interrupt: 3;
    pub u8, dma_setup_fis_interrupt, set_dma_setup_fis_interrupt: 2;
    pub u8, pio_setup_fis_interrupt, set_pio_setup_fis_interrupt: 1;
    pub u8, device_to_host_register_fis_interrupt, set_device_to_host_register_fis_interrupt: 0;
}

impl Clone for AhciPortInterruptStatusRegister {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Debug for AhciPortInterruptStatusRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg = f.debug_struct("AhciPortInterruptStatusRegister");

        if self.cold_port_detect_status() {
            dbg.field("cold_port_detect_status", &true);
        }

        if self.task_file_error_status() {
            dbg.field("task_file_error_status", &true);
        }

        if self.host_bus_fatal_error_status() {
            dbg.field("host_bus_fatal_error_status", &true);
        }

        if self.host_bus_data_error_status() {
            dbg.field("host_bus_data_error_status", &true);
        }

        if self.interface_fatal_error_status() {
            dbg.field("interface_fatal_error_status", &true);
        }

        if self.interface_non_fatal_error_status() {
            dbg.field("interface_non_fatal_error_status", &true);
        }

        if self.overflow_status() {
            dbg.field("overflow_status", &true);
        }

        if self.incorrect_port_multiplier_status() {
            dbg.field("incorrect_port_multiplier_status", &true);
        }

        if self.phyrdy_change_status() {
            dbg.field("phyrdy_change_status", &true);
        }

        if self.device_mechanical_presence_status() {
            dbg.field("device_mechanical_presence_status", &true);
        }

        if self.port_connect_change_status() {
            dbg.field("port_connect_change_status", &true);
        }

        if self.descriptor_processed() {
            dbg.field("descriptor_processed", &true);
        }

        if self.unknown_fis_interrrupt() {
            dbg.field("unknown_fis_interrrupt", &true);
        }

        if self.set_device_bits_interrupt() {
            dbg.field("set_device_bits_interrupt", &true);
        }

        if self.dma_setup_fis_interrupt() {
            dbg.field("dma_setup_fis_interrupt", &true);
        }

        if self.pio_setup_fis_interrupt() {
            dbg.field("pio_setup_fis_interrupt", &true);
        }

        if self.device_to_host_register_fis_interrupt() {
            dbg.field("device_to_host_register_fis_interrupt", &true);
        }

        dbg.finish_non_exhaustive()
    }
}

impl AhciPortInterruptStatusRegister {
    pub fn clear_all(&mut self) {
        self.set_cold_port_detect_status(false);
        self.set_task_file_error_status(false);
        self.set_host_bus_fatal_error_status(false);
        self.set_host_bus_data_error_status(false);
        self.set_interface_fatal_error_status(false);
        self.set_interface_non_fatal_error_status(false);
        self.set_incorrect_port_multiplier_status(false);
        self.set_device_mechanical_presence_status(false);
        self.set_descriptor_processed(false);
        self.set_set_device_bits_interrupt(false);
        self.set_dma_setup_fis_interrupt(false);
        self.set_pio_setup_fis_interrupt(false);
        self.set_device_to_host_register_fis_interrupt(false);
    }
}
