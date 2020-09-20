use neon::prelude::*;

mod ace_serialize;
mod base64;
mod huffman;
mod lib_serialize;

use ace_serialize::{Deserializer as LegacyDeserializer, Serializer};
use lib_serialize::Deserializer;
use std::borrow::Cow;

enum StringVersion {
    Huffman,             // base64
    Deflate,             // '!' + base64
    BinarySerialization, // !WA:\d+! + base64
}

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsValue> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = match cx.argument_opt(1) {
        Some(v) => {
            if v.downcast::<JsUndefined>().is_ok() {
                Ok(8 * 1024 * 1024)
            } else {
                v.downcast_or_throw::<JsNumber, FunctionContext>(&mut cx).and_then(|v| {
                    let v = v.value();
                    if v == f64::INFINITY {
                        Ok(usize::MAX)
                    } else if v.is_finite() {
                        Ok(v.trunc() as usize)
                    } else {
                        cx.throw_type_error("Invalid value, expected a finite number or +Infinity")
                    }
                })
            }
        }
        None => Ok(8 * 1024 * 1024),
    }?;

    let (weakaura, version) = if src.starts_with("!WA:2!") {
        (&src[6..], StringVersion::BinarySerialization)
    } else if src.starts_with('!') {
        (&src[1..], StringVersion::Deflate)
    } else {
        (&src[..], StringVersion::Huffman)
    };

    let decoded = base64::decode(weakaura).unwrap();
    let decompressed = if let StringVersion::Huffman = version {
        huffman::decompress(&decoded, max_size).unwrap()
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
            .unwrap()
    };

    let deserialized = if let StringVersion::BinarySerialization = version {
        Deserializer::from_slice(&decompressed).deserialize_first(&mut cx)
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decompressed)).deserialize_first(&mut cx)
    };

    Ok(deserialized.unwrap())
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let value = cx.argument::<JsValue>(0)?;

    let serialized = Serializer::serialize(&mut cx, value).unwrap();

    let compressed = {
        use flate2::{read::DeflateEncoder, Compression};
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut deflater = DeflateEncoder::new(serialized.as_bytes(), Compression::best());
        deflater.read_to_end(&mut result).unwrap();
        result
    };

    let result = base64::encode_weakaura(&compressed).unwrap();

    Ok(cx.string(result))
}

register_module!(mut m, {
    m.export_function("decode", decode_weakaura)?;
    m.export_function("encode", encode_weakaura)
});
