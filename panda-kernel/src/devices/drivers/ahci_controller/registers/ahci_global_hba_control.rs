// Source, page 8 of:
//   https://www.intel.com/content/www/us/en/io/serial-ata/serial-ata-ahci-spec-rev1-3-1.html

use bitfield::bitfield;

bitfield! {
    pub struct AhciGlobalHbaControlRegister(u32);
    impl Debug;

    pub ahci_enable, set_ahci_enable: 31;
    pub msi_revert_to_single_message, _: 2;
    pub interrupt_enable, set_interrupt_enable: 1;
    pub hba_reset, set_hba_reset: 0;
}
