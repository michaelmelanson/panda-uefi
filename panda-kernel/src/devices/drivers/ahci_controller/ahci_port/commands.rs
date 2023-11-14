use ata::ATACommand;
use thingbuf::mpsc::Sender;

use self::{
    read_connected_status::ReadConnectedStatusReply, send_ata_command::SendATACommandReply,
};

pub mod read_connected_status;
pub mod send_ata_command;

#[derive(Clone, Debug)]
pub enum AhciPortCommand {
    Noop,
    ReadConnectedStatus(Sender<ReadConnectedStatusReply>),
    SendATACommand(ATACommand, Sender<SendATACommandReply>),
}

impl Default for AhciPortCommand {
    fn default() -> Self {
        AhciPortCommand::Noop
    }
}
