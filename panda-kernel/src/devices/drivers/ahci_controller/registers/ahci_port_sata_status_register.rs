// Source, page 29 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

#[derive(Debug, Clone, Copy)]
pub enum AhciPortSataStatusIPM {
    Reserved(u8),
    NotPresent,
    Active,
    Partial,
    Slumber,
    DevSleep,
}

impl From<u8> for AhciPortSataStatusIPM {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Self::NotPresent,
            0x1 => Self::Active,
            0x2 => Self::Partial,
            0x6 => Self::Slumber,
            0x8 => Self::DevSleep,
            x => Self::Reserved(x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AhciPortSataStatusSPD {
    Reserved(u8),
    NotPresent,
    Gen1Rate,
    Gen2Rate,
    Gen3Rate,
}

impl From<u8> for AhciPortSataStatusSPD {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Self::NotPresent,
            0x1 => Self::Gen1Rate,
            0x2 => Self::Gen2Rate,
            0x3 => Self::Gen3Rate,
            x => Self::Reserved(x),
        }
    }
}

bitfield! {
    pub struct AhciPortSataStatusRegister(u32);
    impl Debug;
    u8;

    // 31:12 Reserved
    pub u8, into AhciPortSataStatusIPM, interface_power_mgmt, _: 11, 8;
    pub u8, into AhciPortSataStatusSPD, current_interface_speed, _: 7, 4;
    pub device_detection, _: 3, 0;
}
