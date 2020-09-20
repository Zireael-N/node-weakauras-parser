use neon::prelude::*;

use super::base64;
use super::huffman;

use std::borrow::Cow;

pub fn transform_max_size<'a>(v: Handle<'a, JsValue>, cx: &'a mut FunctionContext) -> NeonResult<Option<usize>> {
    if v.downcast::<JsUndefined>().is_ok() {
        Ok(Some(8 * 1024 * 1024))
    } else {
        v.downcast_or_throw::<JsNumber, FunctionContext>(cx).and_then(|v| {
            let v = v.value();
            if v == f64::INFINITY {
                Ok(None)
            } else if v.is_finite() {
                Ok(Some(v.trunc() as usize))
            } else {
                cx.throw_type_error("Invalid value, expected a finite number or +Infinity")
            }
        })
    }
}

pub fn decode_weakaura(src: &str, max_size: Option<usize>) -> Result<Vec<u8>, &'static str> {
    let (weakaura, legacy) = if src.starts_with('!') {
        (&src[1..], false)
    } else {
        (&src[..], true)
    };

    let decoded = base64::decode(weakaura)?;

    let max_size = max_size.unwrap_or(usize::MAX);
    if legacy {
        huffman::decompress(&decoded, max_size).map(Cow::into_owned)
    } else {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&decoded[..]).take(max_size as u64);

        inflater
            .read_to_end(&mut result)
            .map_err(|_| "Decompression error")
            .and_then(|_| {
                if result.len() < max_size {
                    Ok(result)
                } else {
                    match inflater.into_inner().bytes().next() {
                        Some(_) => Err("Compressed data is too large"),
                        None => Ok(result),
                    }
                }
            })
    }
}

pub fn encode_weakaura(serialized: &str) -> Result<String, &'static str> {
    let compressed = {
        use flate2::{read::DeflateEncoder, Compression};
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut deflater = DeflateEncoder::new(serialized.as_bytes(), Compression::best());

        deflater
            .read_to_end(&mut result)
            .map(|_| result)
            .map_err(|_| "Compression error")
    }?;

    base64::encode_weakaura(&compressed)
}
