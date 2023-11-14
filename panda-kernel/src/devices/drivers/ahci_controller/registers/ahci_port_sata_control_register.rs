// Source, page 8 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

#[allow(unused)]
pub enum DeviceDetectionInit {
    NoOp = 0x0,
    InitCommunication = 0x1,
    DisableInterface = 0x4,
}

bitfield! {
    pub struct AhciPortSataControlRegister(u32);
    impl Debug;

    // 31:20 Reserved
    pub u8, port_multiplier_port, set_port_multiplier_port: 19, 16;
    pub u8, select_power_management, set_select_power_management: 15, 12;
    pub u8, interface_power_mgmt_transitions_allowed, set_interface_power_mgmt_transitions_allowed: 11, 8;
    pub u8, speed_allowed, set_speed_allowed: 7, 4;
    pub u8, device_detection_init, set_device_detection_init: 3, 0;
}
