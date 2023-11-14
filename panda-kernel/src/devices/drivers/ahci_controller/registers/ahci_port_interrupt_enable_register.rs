// Source, page 24-25 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

bitfield! {
    pub struct AhciPortInterruptEnableRegister(u32);
    impl Debug;


    // 31:20 Reserved
    pub u8, cold_presence_detect_enable, set_cold_presence_detect_enable: 31;
    pub u8, task_file_error_enable, set_task_file_error_enable: 30;
    pub u8, host_bus_fatal_error_enable, set_host_bus_fatal_error_enable: 29;
    pub u8, host_bus_data_error_enable, set_host_bus_data_error_enable: 28;
    pub u8, interface_fatal_error_enable, set_interface_fatal_error_enable: 27;
    pub u8, interface_non_fatal_error_enable, set_interface_non_fatal_error_enable: 26;
    // pub u8, reserved_25, set_reserved_25: 25;
    pub u8, overflow_enable, set_overflow_enable: 24;
    pub u8, incorrect_port_multiplier_enable, set_incorrect_port_multiplier_enable: 23;
    pub u8, phyrdy_change_interrupt_enable, set_phyrdy_change_interrupt_enable: 22;
    pub u8, reserved_21_08, set_reserved_21_08: 21, 8;
    pub u8, device_mechanical_presence_enable, set_device_mechanical_presence_enable: 7;
    pub u8, port_change_interupt_enable, set_port_change_interupt_enable: 6;
    pub u8, descriptor_processed_interupt_enable, set_descriptor_processed_interupt_enable: 5;
    pub u8, unknown_fis_interrupt_enable, set_unknown_fis_interrupt_enable: 4;
    pub u8, set_device_bits_interrupt_enable, set_set_device_bits_interrupt_enable: 3;
    pub u8, dma_setup_fis_interrupt_enable, set_dma_setup_fis_interrupt_enable: 2;
    pub u8, pio_setup_fis_interrupt_enable, set_pio_setup_fis_interrupt_enable: 1;
    pub u8, device_to_host_register_fis_interrupt_enable, set_device_to_host_register_fis_interrupt_enable: 0;    
}
