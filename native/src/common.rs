use super::base64;
use super::huffman;

use std::borrow::Cow;

pub fn decode_weakaura(src: &str) -> Result<Vec<u8>, &'static str> {
    let (weakaura, legacy) = if src.starts_with('!') {
        (&src[1..], false)
    } else {
        (&src[..], true)
    };

    let decoded = base64::decode(weakaura)?;

    if legacy {
        huffman::decompress(&decoded).map(Cow::into_owned)
    } else {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&decoded[..]);

        inflater
            .read_to_end(&mut result)
            .map(|_| result)
            .map_err(|_| "compression error")
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
            .map_err(|_| "compression error")
    }?;

    base64::encode_weakaura(&compressed)
}
