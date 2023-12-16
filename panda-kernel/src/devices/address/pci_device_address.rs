#[derive(Debug)]
pub struct PciDeviceAddress {
    segment: u8,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciDeviceAddress {
    pub fn new(segment: u8, bus: u8, device: u8, function: u8) -> PciDeviceAddress {
        Self {
            segment,
            bus,
            device,
            function,
        }
    }
}
