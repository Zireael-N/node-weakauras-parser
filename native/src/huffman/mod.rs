// Code extraction algorithm (GPLv2) is based on code from LibCompress
// Copyright (c) jjsheets and Galmok
// https://www.curseforge.com/wow/addons/libcompress

mod bitfield;
mod lookup_table;
mod utils;

use self::bitfield::Bitfield;
use lookup_table::{build_lookup_table, TableData};
use std::borrow::Cow;
use utils::{get_code, unescape_code};

pub(crate) fn decompress(bytes: &[u8]) -> Result<Cow<'_, [u8]>, &'static str> {
    let mut iter = bytes.iter();
    match iter.next() {
        Some(1) => return Ok(Cow::from(&bytes[1..])),
        Some(3) => (),
        _ => return Err("unknown compression codec"),
    }

    let len = bytes.len();
    if len < 5 {
        return Err("insufficient data");
    }

    let num_symbols = iter.next().unwrap() + 1;
    let original_size = iter
        .by_ref()
        .take(3)
        .map(|&byte| usize::from(byte))
        .enumerate()
        .fold(0, |acc, (i, byte)| acc + (byte << (i * 8)));

    if original_size == 0 {
        return Err("insufficient data");
    }

    let mut codes = Vec::with_capacity(num_symbols as usize);
    let mut result = Vec::with_capacity(original_size);

    let mut bitfield = Bitfield::new();

    let mut min_code_len = std::u8::MAX;
    let mut max_code_len = std::u8::MIN;

    // Code extraction:
    for _ in 0..num_symbols {
        let symbol = bitfield.insert_and_extract_byte(*iter.next().ok_or("unexpected end of input")?);

        loop {
            bitfield
                .insert(*iter.next().ok_or("unexpected end of input")?)
                .map_err(|_| "compression error")?;

            if let Some(v) = get_code(&mut bitfield)? {
                let (code, code_len) = unescape_code(v.0, v.1);
                min_code_len = std::cmp::min(min_code_len, code_len);
                max_code_len = std::cmp::max(max_code_len, code_len);

                codes.push((code, code_len, symbol));

                break;
            }
        }
    }
    codes.sort_unstable_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    // Decompression:
    let lut = build_lookup_table(&codes)?;

    'outer: loop {
        bitfield.fill_from_iterator(&mut iter);

        if bitfield.get_len() >= min_code_len {
            let mut cursor = &lut[(bitfield.peek_byte()) as usize];

            if bitfield.get_len() < cursor.code_length {
                break;
            }

            let mut new_bitfield = bitfield;
            while new_bitfield.get_len() >= cursor.code_length {
                match cursor.data {
                    TableData::Reference(ref v) => {
                        new_bitfield.discard_bits(cursor.code_length);
                        cursor = &v[(new_bitfield.peek_byte()) as usize];
                    }
                    TableData::Symbol(s) => {
                        if cursor.code_length == 0 {
                            if bitfield.get_len() > max_code_len {
                                return Err("compression error");
                            } else {
                                break;
                            }
                        }

                        result.push(s);
                        if result.len() == original_size {
                            break 'outer;
                        }

                        bitfield = new_bitfield;
                        bitfield.discard_bits(cursor.code_length);
                        break;
                    }
                }
            }
        } else {
            break;
        }
    }

    Ok(Cow::from(result))
}
