// Source, page 34 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

#[repr(u8)]
#[derive(Debug)]
#[allow(unused)]
pub enum InterfaceCommunicationControl {
    DevSleep = 0x8,
    Slumber = 0x6,
    Partial = 0x2,
    Active = 0x1,
    NoOpOrIdle = 0x0,
}

bitfield! {
    pub struct AhciCommandAndStatusRegister(u32);
    impl Debug;

    pub u8, icc, set_icc: 31, 28;
    pub aggressive_slumber_partial, set_aggressive_slumber_partial: 27;
    pub aggressive_link_power_mgmt_enable, set_aggressive_link_power_mgmt_enable: 26;
    pub drive_led_on_atapi_enable, set_drive_led_on_atapi_enable: 25;
    pub device_is_atapi, set_device_is_atapi: 24;
    pub auto_partial_to_slumber_transitions_enable, set_auto_partial_to_slumber_transitions_enable: 23;
    pub fis_based_switching_capable_port, _: 22;
    pub external_sata_port, _: 21;
    pub cold_presence_detection, _: 20;
    pub mechanical_presence_switch_attached, _: 19;
    pub hot_plug_capable, _: 18;
    pub port_multiplier_attached, set_port_multiplier_attached: 17;
    pub cold_presence_state, _: 16;
    pub command_list_running, _: 15;
    pub fis_receive_running, _: 14;
    pub mechanical_presence_switch_state, _: 13;
    pub u8, current_command_slot, set_current_command_slot: 12, 8;
    // 7:5 Reserved
    pub fis_receive_enable, set_fis_receive_enable: 4;
    pub command_list_override, set_command_list_override: 3;
    pub power_on_device, set_power_on_device: 2;
    pub spin_up_device, set_spin_up_device: 1;
    pub start, set_start: 0;
}
