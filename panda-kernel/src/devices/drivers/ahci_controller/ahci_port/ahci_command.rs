use bitfield::bitfield;

bitfield! {
    #[derive(Default, Clone, Copy)]
    #[repr(align(1024))]
    pub struct AhciCommandHeader(/* MSB0 */ [u32]);

    // DW 0
    pub u16, phys_region_descriptor_table_length, set_phys_region_descriptor_table_length: 31, 16;
    pub u8, port_multiplier_port, set_port_multiplier_port: 15, 12;
    pub clear_busy_on_r_ok, set_clear_busy_on_r_ok: 10;
    pub bist, set_bist: 9;
    pub reset, set_reset: 8;
    pub prefetchable, set_prefetchable: 7;
    pub write, set_write: 6;
    pub atapi, set_atapi: 5;
    pub u8, command_fis_length, set_command_fis_length: 4, 0;

    // DW 1
    pub u32, phys_region_descriptor_byte_count, set_phys_region_descriptor_byte_count: 63, 32;

    // DW 2
    // Ensure that 06:00 are zero by making this 128-byte aligned
    pub u32, command_table_descriptor_base_address_lower, set_command_table_descriptor_base_address_lower: 95, 64;

    // DW 3
    pub u32, command_table_descriptor_base_address_upper, set_command_table_descriptor_base_address_upper: 127, 96;
}

#[repr(C, align(1024))]
#[derive(Default, Clone, Copy)]
pub struct AhciCommandList(pub [AhciCommandHeader<[u32; 8]>; 10]);
