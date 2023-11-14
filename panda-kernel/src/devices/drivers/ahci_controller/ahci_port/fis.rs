use core::mem::ManuallyDrop;

pub use self::{
    device_to_host_fis::DeviceToHostRegisterFis, host_to_device_fis::HostToDeviceRegisterFis,
};

mod device_to_host_fis;
mod host_to_device_fis;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[allow(unused)]
pub enum FisType {
    Unknown = 0x00,

    HostToDeviceRegisterFis = 0x27, // Register FIS - host to device
    DeviceToHostRegisterFis = 0x34, // Register FIS - device to host
    ActivateDma = 0x39,             // DMA activate FIS - device to host
    SetupDma = 0x41,                // DMA setup FIS - bidirectional
    Data = 0x46,                    // Data FIS - bidirectional
    BuiltInSelfTest = 0x58,         // BIST activate FIS - bidirectional
    PioSetup = 0x5F,                // PIO setup FIS - device to host
    SetDeviceBits = 0xA1,           // Set device bits FIS - device to host
}

#[repr(C, align(256))]
pub union AhciFis {
    raw: [u8; 64],
    fis_type: FisType, // for constructing & checking type

    d2h_fis: ManuallyDrop<DeviceToHostRegisterFis<[u8; 64]>>,
    h2d_fis: ManuallyDrop<HostToDeviceRegisterFis<[u8; 64]>>,
}

impl Default for AhciFis {
    fn default() -> Self {
        Self {
            fis_type: FisType::Unknown,
        }
    }
}

impl AhciFis {
    pub fn fis_type(&self) -> FisType {
        // safe because all the subtypes have this field
        unsafe { self.fis_type }
    }

    pub fn to_d2h_fis(&self) -> Option<&DeviceToHostRegisterFis<[u8; 64]>> {
        match self.fis_type() {
            FisType::DeviceToHostRegisterFis => unsafe { Some(&self.d2h_fis) },
            _ => None,
        }
    }

    pub fn to_h2d_fis(&self) -> Option<&HostToDeviceRegisterFis<[u8; 64]>> {
        match self.fis_type() {
            FisType::HostToDeviceRegisterFis => unsafe { Some(&self.h2d_fis) },
            _ => None,
        }
    }
}

#[test]
fn test_correct_size() {
    assert_eq!(core::mem::size_of::<AhciFis>(), 64);
}
