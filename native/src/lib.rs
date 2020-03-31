use neon::prelude::*;

mod base64;
mod deserialization;
mod huffman;
mod serialization;

mod asynchronous;
mod common;
mod synchronous;

register_module!(mut m, {
    m.export_function("decode", asynchronous::decode_weakaura)?;
    m.export_function("encode", asynchronous::encode_weakaura)?;
    m.export_function("decodeSync", synchronous::decode_weakaura)?;
    m.export_function("encodeSync", synchronous::encode_weakaura)
});
