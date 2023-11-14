use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct ReadCommand {
    start_sector: u64,
    sector_count: u64,
}

#[derive(Debug)]
pub struct ReadCommandReply {
    data: Vec<SectorData>,
}

type SectorData = Vec<u8>;
