// Source, page 8 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

bitfield! {
    pub struct AhciPciCommandRegister(u16);
    impl Debug;

    pub interrupt_disable, set_interrupt_disable: 10;
    pub fast_back_to_back_enable, set_fast_back_to_back_enable: 9;
    pub serr, set_serr: 8;
    pub wait_cycle_enable, _: 7;
    pub parity_error_response_enable, set_parity_error_response_enable: 6;
    pub vga_palette_snoop_enable, _: 5;
    pub memory_write_and_invalidate_enable, set_memory_write_and_invalidate_enable: 4;
    pub special_cycle_enable, _: 3;
    pub bus_master_enable, set_bus_master_enable: 2;
    pub memory_space_enable, set_memory_space_enable: 1;
    pub io_space_enable, set_io_space_enable: 0;
}
