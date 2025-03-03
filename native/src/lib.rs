use neon::prelude::*;

mod ace_serialize;
mod base64;
mod huffman;
mod lib_serialize;
mod macros;

mod asynchronous;
mod common;
mod synchronous;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("decode", asynchronous::decode_weakaura)?;
    cx.export_function("encode", asynchronous::encode_weakaura)?;
    cx.export_function("decodeSync", synchronous::decode_weakaura)?;
    cx.export_function("encodeSync", synchronous::encode_weakaura)?;

    Ok(())
}
