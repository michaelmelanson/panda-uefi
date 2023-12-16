//ew
use crate::devices::drivers::ahci_controller::ahci_port::ide::ide_identify::IdeIdentifyData;

#[derive(Default, Clone, Debug)]
pub struct IdentifyCommandReply {
    pub ide_identify: IdeIdentifyData,
}
