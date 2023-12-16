use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct ReadCommand {
    pub start_sector: u64,
    pub sector_count: u64,
}

#[derive(Debug, Default, Clone)]
pub struct ReadCommandReply {
    pub data: Vec<SectorData>,
}

type SectorData = [u8; 512];
