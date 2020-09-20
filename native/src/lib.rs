use neon::prelude::*;

mod ace_serialize;
mod base64;
mod huffman;

mod asynchronous;
mod common;
mod synchronous;

register_module!(mut m, {
    m.export_function("decode", asynchronous::decode_weakaura)?;
    m.export_function("encode", asynchronous::encode_weakaura)?;
    m.export_function("decodeSync", synchronous::decode_weakaura)?;
    m.export_function("encodeSync", synchronous::encode_weakaura)
});
