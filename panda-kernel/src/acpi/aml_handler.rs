use core::fmt::{Display, UpperHex};

use x86_64::{
    instructions::port::Port,
    structures::port::{PortRead, PortWrite},
    PhysAddr,
};

use crate::{memory, pci};

pub struct AmlHandler {}

impl AmlHandler {
    pub fn new() -> Self {
        AmlHandler {}
    }

    fn read<T: UpperHex + Copy>(&self, address: usize) -> T {
        let addr = memory::physical_to_virtual(PhysAddr::new(address as u64));

        let value = unsafe { *addr.as_ptr() };
        log::info!("AML: memory read from {address:#X} = {value:#X}");
        value
    }

    fn pci_read<T: 'static + UpperHex + Copy + Default>(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
    ) -> T {
        match pci::read(segment, bus, device, function, offset) {
            Ok(value) => {
                log::info!("AML: PCI read from {segment:X}:{bus:X}:{device:X}:{function:X}+{offset:X} = {value:X}");
                value
            }

            Err(err) => {
                log::error!("AML: PCI read error: {err:?}");
                T::default()
            }
        }
    }

    fn io_read<T: Display + PortRead>(&self, port_num: u16) -> T {
        let mut port = Port::new(port_num);
        let value = unsafe { port.read() };

        log::info!("AML: IO read from port {port_num:X} = {value}");
        value
    }

    fn io_write<T: Copy + Display + PortWrite>(&self, port_num: u16, value: T) {
        let mut port = Port::new(port_num);
        unsafe { port.write(value) };

        log::info!("AML: IO write to port {port_num:X} = {value}");
    }
}

impl aml::Handler for AmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        self.read(address)
    }

    fn read_u16(&self, address: usize) -> u16 {
        self.read(address)
    }

    fn read_u32(&self, address: usize) -> u32 {
        self.read(address)
    }

    fn read_u64(&self, address: usize) -> u64 {
        self.read(address)
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        todo!("aml write_u8 at {address:X} with value {value:X}")
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        todo!("aml write_u16 at {address:X} with value {value:X}")
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        todo!("aml write_u32 at {address:X} with value {value:X}")
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        todo!("aml write_u64 at {address:X} with value {value:X}")
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        self.io_read(port)
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        self.io_read(port)
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        self.io_read(port)
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        self.io_write(port, value)
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        self.io_write(port, value)
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        self.io_write(port, value)
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        self.pci_read(segment, bus, device, function, offset)
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        self.pci_read(segment, bus, device, function, offset)
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        self.pci_read(segment, bus, device, function, offset)
    }

    fn write_pci_u8(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u8,
    ) {
        todo!("aml write_pci_u8 at {segment:X}:{bus:X}:{device:X}:{function:X}:{offset:X} with value {value:X}")
    }

    fn write_pci_u16(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u16,
    ) {
        todo!("aml write_pci_u16 at {segment:X}:{bus:X}:{device:X}:{function:X}:{offset:X} with value {value:X}")
    }

    fn write_pci_u32(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u32,
    ) {
        todo!("aml write_pci_u32 at {segment:X}:{bus:X}:{device:X}:{function:X}:{offset:X} with value {value:X}")
    }

    fn stall(&self, microseconds: u64) {
        todo!("aml stall {microseconds}µs");
    }

    fn sleep(&self, milliseconds: u64) {
        todo!("aml sleep {milliseconds}ms");
    }
}
