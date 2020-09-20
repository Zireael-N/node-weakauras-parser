use neon::prelude::*;

use std::borrow::Cow;

use super::base64;
use super::huffman;

use super::ace_serialize::{Deserializer as LegacyDeserializer, Serializer};
use super::lib_serialize::Deserializer;

enum StringVersion {
    Huffman,             // base64
    Deflate,             // '!' + base64
    BinarySerialization, // !WA:\d+! + base64
}

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

pub fn decode_weakaura(src: &str, max_size: Option<usize>) -> Result<String, &'static str> {
    let (weakaura, version) = if src.starts_with("!WA:2!") {
        (&src[6..], StringVersion::BinarySerialization)
    } else if src.starts_with('!') {
        (&src[1..], StringVersion::Deflate)
    } else {
        (&src[..], StringVersion::Huffman)
    };

    let decoded = base64::decode(weakaura)?;

    let max_size = max_size.unwrap_or(usize::MAX);
    let decompressed = if let StringVersion::Huffman = version {
        huffman::decompress(&decoded, max_size)
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
                    Ok(())
                } else {
                    match inflater.into_inner().bytes().next() {
                        Some(_) => Err("Compressed data is too large"),
                        None => Ok(()),
                    }
                }
            })
            .map(|_| Cow::from(result))
    }?;

    let deserialized = if let StringVersion::BinarySerialization = version {
        Deserializer::from_slice(&decompressed).deserialize_first()
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decompressed)).deserialize_first()
    }?;

    serde_json::to_string(&deserialized).map_err(|_| "Failed to convert to JSON")
}

pub fn encode_weakaura(json: &str) -> Result<String, &'static str> {
    let serialized = serde_json::from_str(&json)
        .map_err(|_| "Failed to parse JSON")
        .and_then(|val| Serializer::serialize(&val, Some(json.len())))?;

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
