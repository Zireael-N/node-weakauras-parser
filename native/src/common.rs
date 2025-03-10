use neon::prelude::*;

use std::borrow::Cow;

use super::base64;
use super::huffman;

use super::ace_serialize::{Deserializer as LegacyDeserializer, Serializer as LegacySerializer};
use super::lib_serialize::{Deserializer, Serializer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StringVersion {
    Huffman,             // base64
    Deflate,             // '!' + base64
    BinarySerialization, // !WA:\d+! + base64
}

pub fn parse_max_size<'a>(
    v: Option<Handle<'a, JsValue>>,
    cx: &'a mut FunctionContext,
) -> NeonResult<Option<usize>> {
    match v {
        Some(v) => {
            if v.is_a::<JsUndefined, _>(cx) {
                Ok(Some(8 * 1024 * 1024))
            } else {
                v.downcast_or_throw::<JsNumber, _>(cx).and_then(|v| {
                    let v = v.value(cx);
                    if v == f64::INFINITY {
                        Ok(None)
                    } else if v.is_finite() && v >= 0.0 {
                        Ok(Some(v.trunc() as usize))
                    } else {
                        cx.throw_type_error(
                            "Invalid value, expected a positive finite number or +Infinity",
                        )
                    }
                })
            }
        }
        None => Ok(Some(8 * 1024 * 1024)),
    }
}

pub fn parse_string_version<'a>(
    v: Option<Handle<'a, JsValue>>,
    cx: &'a mut FunctionContext,
) -> NeonResult<StringVersion> {
    match v {
        Some(v) => {
            if v.is_a::<JsUndefined, _>(cx) {
                Ok(StringVersion::BinarySerialization)
            } else {
                v.downcast_or_throw::<JsNumber, _>(cx).and_then(|v| {
                    let v = v.value(cx).trunc() as u64;
                    match v {
                        1 => Ok(StringVersion::Deflate),
                        2 => Ok(StringVersion::BinarySerialization),
                        _ => cx.throw_type_error("Invalid value"),
                    }
                })
            }
        }
        None => Ok(StringVersion::BinarySerialization),
    }
}

pub fn decode_weakaura(src: &str, max_size: Option<usize>) -> Result<String, &'static str> {
    let (weakaura, version) = if let Some(src) = src.strip_prefix("!WA:2!") {
        (src, StringVersion::BinarySerialization)
    } else if let Some(src) = src.strip_prefix('!') {
        (src, StringVersion::Deflate)
    } else {
        (src, StringVersion::Huffman)
    };

    let decoded = base64::decode(weakaura.trim_ascii_end())?;

    let max_size = max_size.unwrap_or(usize::MAX);
    let decompressed = if version == StringVersion::Huffman {
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

    let deserialized = if version == StringVersion::BinarySerialization {
        Deserializer::from_slice(&decompressed).deserialize_first()
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decompressed)).deserialize_first()
    }?;

    serde_json::to_string(&deserialized).map_err(|_| "Failed to convert to JSON")
}

pub fn encode_weakaura(json: &str, format: StringVersion) -> Result<String, &'static str> {
    let data = serde_json::from_str(json).map_err(|_| "Failed to parse JSON")?;

    let (serialized, prefix) = match format {
        StringVersion::Deflate => (
            LegacySerializer::serialize(&data, Some(json.len())).map(|v| v.into_bytes())?,
            "!",
        ),
        StringVersion::BinarySerialization => {
            (Serializer::serialize(data, Some(json.len()))?, "!WA:2!")
        }
        _ => return Err("Unsupported StringVersion"),
    };

    let compressed = {
        use flate2::{Compression, read::DeflateEncoder};
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut deflater = DeflateEncoder::new(serialized.as_slice(), Compression::best());

        deflater
            .read_to_end(&mut result)
            .map(|_| result)
            .map_err(|_| "Compression error")
    }?;

    base64::encode_with_prefix(&compressed, prefix)
}
