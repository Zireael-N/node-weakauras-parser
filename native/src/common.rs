use std::borrow::Cow;

use super::base64;
use super::huffman;

use super::deserialization::Deserializer;
use super::serialization::Serializer;

pub fn decode_weakaura(src: &str) -> Result<String, &'static str> {
    let (weakaura, legacy) = if src.starts_with('!') {
        (&src[1..], false)
    } else {
        (&src[..], true)
    };

    let decoded = base64::decode(weakaura)?;

    let decompressed = if legacy {
        huffman::decompress(&decoded)
    } else {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&decoded[..]);

        inflater
            .read_to_end(&mut result)
            .map(|_| Cow::from(result))
            .map_err(|_| "compression error")
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
