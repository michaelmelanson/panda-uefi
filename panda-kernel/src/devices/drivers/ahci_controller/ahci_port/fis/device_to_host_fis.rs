use bitfield::bitfield;

use super::FisType;

bitfield! {
    pub struct DeviceToHostRegisterFis([u8]);
    impl Debug;
    u8;

    // DWORD 0
    fis_type, set_fis_type: 7, 0;
    pub pmport, set_pmport: 11, 8;
    pub interrrupt_bit, set_interrupt_bit: 14;
    pub status, set_status: 23, 16;
    pub error, set_error: 31, 24;

    // DWORD 1
    lba_low, set_lba_low: 55, 32;        // LBA low register, 23:0
    pub device, set_device: 63, 56;      // Device register

    // DWORD 2
    lba_high, set_lba_high: 87, 64;        // LBA register, 47:24
    // 8 bits reserved

    // DWORD 3
    pub u16, count: 111, 96;      // Count register
    // 16 bits reserved

    // DWORD 4
    // 32 bits reserved
}


// impl<T: AsMut<[u8]>> DeviceToHostRegisterFis<T> {}
