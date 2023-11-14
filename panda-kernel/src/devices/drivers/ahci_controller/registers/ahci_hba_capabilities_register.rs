// Source, page 8 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

bitfield! {
    pub struct AhciHbaCapabilitiesRegister(u32);
    impl Debug;

    pub supports_64bit_addressing, _: 31;
    pub supports_native_command_queuing, _: 30;
    pub supports_notification_register, _: 29;
    pub supports_mechanical_presence_switch, _: 28;
    pub supports_staggered_spinup, _: 27;
    pub supports_aggressive_link_power_management, _: 26;
    pub supports_activity_led, _: 25;
    pub supports_command_list_override, _: 24;
    pub u8, interface_speed_support, _: 23, 20;
    pub supports_ahci_mode_only, _: 18;
    pub supports_port_multiplier, _: 17;
    pub supports_fis_based_switching, _: 16;
    pub supports_pio_multiple_drq_block, _: 15;
    pub supports_slumber_state, _: 14;
    pub supports_partial_state, _: 13;
    pub u8, number_of_command_slots, _: 12, 8;
    pub supports_command_completion_coalescing, _: 7;
    pub supports_enclosure_management, _: 6;
    pub supports_external_sata, _: 5;
    pub u8, number_of_ports, _: 4, 0;
}
