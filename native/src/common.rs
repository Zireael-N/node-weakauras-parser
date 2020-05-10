use neon::prelude::*;

use std::borrow::Cow;

use super::base64;
use super::huffman;

use super::deserialization::Deserializer;
use super::serialization::Serializer;

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
                cx.throw_type_error("invalid value, expected a finite number or +Infinity")
            }
        })
    }
}

pub fn decode_weakaura(src: &str, max_size: Option<usize>) -> Result<String, &'static str> {
    let (weakaura, legacy) = if src.starts_with('!') {
        (&src[1..], false)
    } else {
        (&src[..], true)
    };

    let decoded = base64::decode(weakaura)?;

    let max_size = max_size.unwrap_or(usize::MAX);
    let decompressed = if legacy {
        huffman::decompress(&decoded, max_size)
    } else {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&decoded[..]).take(max_size as u64);

        inflater
            .read_to_end(&mut result)
            .map_err(|_| "decompression error")
            .and_then(|_| {
                if result.len() < max_size {
                    Ok(())
                } else {
                    match inflater.into_inner().bytes().next() {
                        Some(_) => Err("compressed data is too large"),
                        None => Ok(()),
                    }
                }
            })
            .map(|_| Cow::from(result))
    }?;

    Deserializer::from_str(&String::from_utf8_lossy(&decompressed))
        .deserialize_first()
        .and_then(|deserialized| serde_json::to_string(&deserialized).map_err(|_| "failed to convert to JSON"))
}

pub fn encode_weakaura(json: &str) -> Result<String, &'static str> {
    let serialized = serde_json::from_str(&json)
        .map_err(|_| "failed to parse JSON")
        .and_then(|val| Serializer::serialize(&val, Some(json.len())))?;

    let compressed = {
        use flate2::{read::DeflateEncoder, Compression};
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut deflater = DeflateEncoder::new(serialized.as_bytes(), Compression::best());

        deflater
            .read_to_end(&mut result)
            .map(|_| result)
            .map_err(|_| "compression error")
    }?;

    base64::encode_weakaura(&compressed)
}
