#![no_std]

extern crate alloc;

use alloc::string::String;

const HEX_ASC: &'static str = "0123456789ABCDEF";

fn hex_asc_lo(x: u32) -> char {
    HEX_ASC.chars().nth((x & 0x0f) as usize).unwrap()
}

fn hex_asc_hi(x: u32) -> char {
    HEX_ASC.chars().nth(((x & 0xf0) >> 4) as usize).unwrap()
}

fn nth_uppercase_letter(x: u32) -> char {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    letters.chars().nth(x as usize).unwrap()
}


pub fn decode_eisa_id(id: u32) -> String {
    let id = u32::from_be(id);
    
    let mut result = String::with_capacity(7);
    result.push(nth_uppercase_letter(((id >> 26) & 0b111111) - 1));
    result.push(nth_uppercase_letter(((id >> 21) & 0b011111) - 1));
    result.push(nth_uppercase_letter(((id >> 16) & 0b011111) - 1));
    result.push(hex_asc_hi(id >> 8));
    result.push(hex_asc_lo(id >> 8));
    result.push(hex_asc_hi(id));
    result.push(hex_asc_lo(id));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_asc_lo() {
        assert_eq!(hex_asc_lo(0xf0), '0');
        assert_eq!(hex_asc_lo(0xf1), '1');
        assert_eq!(hex_asc_lo(0xf2), '2');
        assert_eq!(hex_asc_lo(0xf3), '3');
        assert_eq!(hex_asc_lo(0xf4), '4');
        assert_eq!(hex_asc_lo(0xf5), '5');
        assert_eq!(hex_asc_lo(0xf6), '6');
        assert_eq!(hex_asc_lo(0xf7), '7');
        assert_eq!(hex_asc_lo(0xff), 'F');
    }
    

    #[test]
    fn test_hex_asc_hi() {
        assert_eq!(hex_asc_hi(0x0f), '0');
        assert_eq!(hex_asc_hi(0x1f), '1');
        assert_eq!(hex_asc_hi(0x2f), '2');
        assert_eq!(hex_asc_hi(0x3f), '3');
        assert_eq!(hex_asc_hi(0x4f), '4');
        assert_eq!(hex_asc_hi(0x5f), '5');
        assert_eq!(hex_asc_hi(0x6f), '6');
        assert_eq!(hex_asc_hi(0x7f), '7');
        assert_eq!(hex_asc_hi(0xf0), 'F');
    }

    #[test]
    fn test_decode_eisa_id() {
        assert_eq!(decode_eisa_id(0x105D041), String::from("PNP0501"));
        assert_eq!(decode_eisa_id(0x301D041), String::from("PNP0103"));
        assert_eq!(decode_eisa_id(0xF0CD041), String::from("PNP0C0F"));

    }
}
