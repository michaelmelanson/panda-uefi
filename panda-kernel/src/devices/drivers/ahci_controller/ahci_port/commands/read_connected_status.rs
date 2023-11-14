use crate::devices::drivers::ahci_controller::registers::{
    AhciPortSataStatusIPM, AhciPortSataStatusSPD,
};

#[derive(Clone, Debug)]
pub enum ReadConnectedStatusReply {
    Unknown,
    Connected {
        interface_speed: AhciPortSataStatusSPD,
        power_state: AhciPortSataStatusIPM,
    },
    Disconnected,
    NotPresent,
    OfflineMode,
}

impl Default for ReadConnectedStatusReply {
    fn default() -> Self {
        ReadConnectedStatusReply::Unknown
    }
}
