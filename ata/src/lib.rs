#![no_std]

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum ATACommand {
    Identify,
}

// One source I found: https://dox.ipxe.org/ata_8h.html
impl Into<u8> for ATACommand {
    fn into(self) -> u8 {
        match self {
            ATACommand::Identify => 0xEC,
        }
    }
}
