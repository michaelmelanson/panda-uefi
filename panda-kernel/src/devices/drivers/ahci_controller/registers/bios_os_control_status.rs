use bitfield::bitfield;

bitfield! {
    pub struct AhciBiosOsControlStatusRegister(u32);
    impl Debug;

    pub bios_busy, _: 4;
    pub os_ownership_change, _: 3;
    pub smi_on_ooc, _: 2;
    pub oos, set_oos: 1;
    pub bos, set_bos: 0;
}
