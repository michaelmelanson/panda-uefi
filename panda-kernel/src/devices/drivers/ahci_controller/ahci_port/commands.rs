use alloc::{boxed::Box, vec::Vec};
use thingbuf::mpsc::Sender;

use self::{
    identify::IdentifyCommandReply,
    read::{ReadCommand, ReadCommandReply},
    read_connected_status::ReadConnectedStatusReply,
};

use super::ide::ide_identify::IdeIdentifyData;

pub mod identify;
pub mod read;
pub mod read_connected_status;

#[derive(Clone, Debug)]
pub enum AhciPortCommand {
    Noop,
    ReadConnectedStatus(Sender<ReadConnectedStatusReply>),
    Identify(Sender<IdentifyCommandReply>),
    Read(ReadCommand, Sender<ReadCommandReply>),
}

impl Default for AhciPortCommand {
    fn default() -> Self {
        AhciPortCommand::Noop
    }
}
