// Source, page 29 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

bitfield! {
    pub struct AhciPortTaskFileDataRegister(u32);
    impl Debug;
    u8;

    // 31:16 Reserved
    pub error, _: 15, 8;

    pub status, _: 7, 0;
    pub status_busy, _: 7;
    pub command_specific_high, _: 6, 4;
    pub status_data_requested, _: 3;
    pub command_specific_low, _: 2, 1;
    pub status_error, _: 0;
}
