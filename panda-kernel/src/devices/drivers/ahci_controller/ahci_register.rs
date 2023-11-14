
#[allow(dead_code)]
pub enum AhciRegister {
    HostCapability,
    GlobalHostControl,
    InterruptStatus,
    PortImplemented,
    Version,
    CommandCompletionCoalescingControl,
    CommandCompletionCoalescingPorts,
    EnclosureManagementLocation,
    EnclosureManagementControl,
    HostCapabilitiesExtended,
    BIOSOSHandoffControlAndStatus,
    // Vendor and Ports handled separately
}

impl AhciRegister {
    pub fn offset(&self) -> usize {
        match self {
            AhciRegister::HostCapability => 0x00,
            AhciRegister::GlobalHostControl => 0x04,
            AhciRegister::InterruptStatus => 0x08,
            AhciRegister::PortImplemented => 0x0C,
            AhciRegister::Version => 0x10,
            AhciRegister::CommandCompletionCoalescingControl => 0x14,
            AhciRegister::CommandCompletionCoalescingPorts => 0x18,
            AhciRegister::EnclosureManagementLocation => 0x1C,
            AhciRegister::EnclosureManagementControl => 0x20,
            AhciRegister::HostCapabilitiesExtended => 0x24,
            AhciRegister::BIOSOSHandoffControlAndStatus => 0x28,
        }
    }
}
