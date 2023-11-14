#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum AhciPortRegister {
    CommandListBaseAddress = 0x00,      // 1K-byte aligned,
    CommandListBaseAddressUpper = 0x04, // 32 bits
    FISBaseAddress = 0x08,              // 256-byte aligned
    FISBaseAddressUpper = 0x0C,         // 32 bits
    InterruptStatus = 0x10,
    InterruptEnable = 0x14,
    CommandAndStatus = 0x18,
    // Reserved = 0x1C,
    TaskFileData = 0x20,
    Signature = 0x24,
    SATAStatus = 0x28,  // (SCR0:SStatus),
    SATAControl = 0x2C, // (SCR2:SControl),
    SATAError = 0x30,   // (SCR1:SError),
    SATAActive = 0x34,  // (SCR3:SActive),
    CommandIssue = 0x38,
    SATANotification = 0x3C,
    FISBasedSwitchControl = 0x40,
    DeviceSleep = 0x44,
    // Reserved and vendor specific handled separately
}

impl AhciPortRegister {
    pub fn offset(&self) -> usize {
        match self {
            AhciPortRegister::CommandListBaseAddress => 0x00,
            AhciPortRegister::CommandListBaseAddressUpper => 0x04,
            AhciPortRegister::FISBaseAddress => 0x08,
            AhciPortRegister::FISBaseAddressUpper => 0x0c,
            AhciPortRegister::InterruptStatus => 0x10,
            AhciPortRegister::InterruptEnable => 0x14,
            AhciPortRegister::CommandAndStatus => 0x18,
            // Reserved
            AhciPortRegister::TaskFileData => 0x20,
            AhciPortRegister::Signature => 0x24,
            AhciPortRegister::SATAStatus => 0x28,
            AhciPortRegister::SATAControl => 0x2c,
            AhciPortRegister::SATAError => 0x30,
            AhciPortRegister::SATAActive => 0x34,
            AhciPortRegister::CommandIssue => 0x38,
            AhciPortRegister::SATANotification => 0x3c,
            AhciPortRegister::FISBasedSwitchControl => 0x40,
            AhciPortRegister::DeviceSleep => 0x44,
        }
    }
}
