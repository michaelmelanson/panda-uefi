use core::default::Default;

use bitfield::bitfield;

use super::FisType;

bitfield! {
    pub struct HostToDeviceRegisterFis([u8]);
    u8;

    // DWORD 0
    fis_type, set_fis_type: 7, 0;

    pub pmport, set_pmport: 11, 8;
    pub command_or_control, set_command_or_control: 15; // 1 = command, 0 = control

    pub command, set_command: 23, 16; // Command register
    feature_low, set_feature_low: 31, 24; // Feature register, 7:0

    // DWORD 1
    lba0, set_lba0: 39, 32; // LBA low register, 7:0
    lba1, set_lba1: 47, 40; // LBA mid register, 15:8
    lba2, set_lba2: 55, 48; // LBA high register, 23:16
    pub device, set_device: 63, 56; // Device register

    // DWORD 2
    lba3, set_lba3: 71, 64; // LBA register, 31:24
    lba4, set_lba4: 79, 72; // LBA register, 39:32
    lba5, set_lba5: 87, 80; // LBA register, 47:40
    feature_high, set_feature_high:	95, 88; // Feature register, 15:8

    // DWORD 3
    count_low, set_count_low: 103, 96; // Count register, 7:0
    count_high, set_count_high: 111, 104; // Count register, 15:8
    pub icc, set_icc: 119, 112;		// Isochronous command completion
    pub control, set_control: 127, 120;	// Control register
}

impl<T> Default for HostToDeviceRegisterFis<T>
where
    T: AsMut<[u8]> + Default,
{
    fn default() -> Self {
        let mut fis = Self(Default::default());
        fis.set_fis_type(FisType::HostToDeviceRegisterFis as u8);
        fis
    }
}

impl<T: AsMut<[u8]>> HostToDeviceRegisterFis<T> {
    pub fn new(value: T) -> Self {
        let mut s = Self(value);
        s.set_fis_type(FisType::HostToDeviceRegisterFis as u8);
        s
    }

    pub fn set_lba(&mut self, lba: u64) {
        self.set_lba0(((lba >> 0) & 0xff) as u8);
        self.set_lba1(((lba >> 8) & 0xff) as u8);
        self.set_lba2(((lba >> 16) & 0xff) as u8);
        self.set_lba3(((lba >> 24) & 0xff) as u8);
        self.set_lba4(((lba >> 32) & 0xff) as u8);
        self.set_lba5(((lba >> 40) & 0xff) as u8);
    }

    pub fn set_count(&mut self, count: u16) {
        self.set_count_low(((count >> 0) & 0xff) as u8);
        self.set_count_high(((count >> 8) & 0xff) as u8);
    }

    pub fn set_feature(&mut self, feature: u16) {
        self.set_feature_low(((feature >> 0) & 0xff) as u8);
        self.set_feature_high(((feature >> 8) & 0xff) as u8);
    }
}
