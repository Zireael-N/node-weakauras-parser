// Based on code from LibCompress
// Copyright (c) jjsheets and Galmok
// https://www.curseforge.com/wow/addons/libcompress
// Licensed under GPLv2 (see LICENSES/LibCompress)

use super::bitfield::Bitfield;

pub(crate) fn get_code(bitfield: &mut Bitfield) -> Result<Option<(u32, u8)>, &'static str> {
    if bitfield.get_len() >= 2 {
        for i in 0..=bitfield.get_len() - 2 {
            let b1 = bitfield.get_data() & (1 << i);
            let b2 = bitfield.get_data() & (1 << (i + 1));
            if b1 != 0 && b2 != 0 {
                return if i <= 32 {
                    let code = bitfield.extract_bits(i) as u32;
                    bitfield.discard_bits(2);
                    Ok(Some((code, i)))
                } else {
                    Err("Unsupported code length")
                };
            }
        }
    }
    Ok(None)
}

pub(crate) fn unescape_code(code: u32, code_len: u8) -> (u32, u8) {
    let mut unescaped_code: u32 = 0;
    let mut i: u8 = 0;
    let mut l: u8 = 0;
    while i < code_len {
        if (code & (1 << (i as i32))) != 0 {
            unescaped_code |= 1 << (l as u32);
            i += 1;
        }
        i += 1;
        l += 1;
    }
    (unescaped_code, l)
}
