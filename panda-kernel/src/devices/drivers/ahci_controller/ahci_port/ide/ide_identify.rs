use core::fmt::Debug;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone)]
#[repr(C)]
pub struct IdeIdentifyData {
    pub signature: u16,             // 0: 0x0040
    pub cylinders: u16,             // 1: put_le16(p + 1, s->cylinders);
    reserved0: u16,                 // 2:
    pub heads: u16,                 // 3: put_le16(p + 3, s->heads);
    reserved1: u16, // 4: put_le16(p + 4, 512 * s->sectors); /* XXX: retired, remove ? */
    reserved2: u16, // 5: put_le16(p + 5, 512); /* XXX: retired, remove ? */
    pub sectors: u16, // 6: put_le16(p + 6, s->sectors);
    reserved3: u16, // 7
    reserved4: u16, // 8
    reserved5: u16, // 9
    pub serial_number: [u16; 10], // 10-19: padstr((char *)(p + 10), s->drive_serial_str, 20); /* serial number */
    reserved6: u16,               // 20: put_le16(p + 20, 3); /* XXX: retired, remove ? */
    pub cache_size_in_sectors: u16, // 21: put_le16(p + 21, 512); /* cache size in sectors */
    pub ecc_bytes: u16,           // 22: put_le16(p + 22, 4); /* ecc bytes */
    pub firmware_version: [u16; 4], // 23-26: padstr((char *)(p + 23), s->version, 8); /* firmware version */
    pub model: [u16; 20], // 26-45: padstr((char *)(p + 27), s->drive_model_str, 40); /* model */

    reserved7: [u8; 418],
}

impl Default for IdeIdentifyData {
    fn default() -> Self {
        Self {
            signature: Default::default(),
            cylinders: Default::default(),
            reserved0: Default::default(),
            heads: Default::default(),
            reserved1: Default::default(),
            reserved2: Default::default(),
            sectors: Default::default(),
            reserved3: Default::default(),
            reserved4: Default::default(),
            reserved5: Default::default(),
            serial_number: Default::default(),
            reserved6: Default::default(),
            cache_size_in_sectors: Default::default(),
            ecc_bytes: Default::default(),
            firmware_version: Default::default(),
            model: Default::default(),
            reserved7: [0; 418],
        }
    }
}

impl Debug for IdeIdentifyData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("IdeIdentifyData")
            .field("signature", &self.signature)
            .field("cylinders", &self.cylinders)
            // .field("reserved0", &self.reserved0)
            .field("heads", &self.heads)
            // .field("reserved1", &self.reserved1)
            // .field("reserved2", &self.reserved2)
            .field("sectors", &self.sectors)
            // .field("reserved3", &self.reserved3)
            // .field("reserved4", &self.reserved4)
            // .field("reserved5", &self.reserved5)
            .field("serial_number", &utf16array_to_string(&self.serial_number))
            // .field("reserved6", &self.reserved6)
            .field("cache_size_in_sectors", &self.cache_size_in_sectors)
            .field("ecc_bytes", &self.ecc_bytes)
            .field(
                "firmware_version",
                &utf16array_to_string(&self.firmware_version),
            )
            .field("model", &utf16array_to_string(&self.model))
            .finish()
    }
}

fn utf16array_to_string(data: &[u16]) -> String {
    let mut bytes = Vec::new();
    bytes.reserve(data.len() / 2);

    for word in data {
        bytes.extend_from_slice(&word.to_be_bytes());
    }

    core::str::from_utf8(&bytes).unwrap().trim().to_string()
}
