use neon::prelude::*;

use super::common;
use super::ace_serialize::{Deserializer, Serializer};

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsValue> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = match cx.argument_opt(1) {
        Some(v) => common::transform_max_size(v, &mut cx),
        None => Ok(Some(8 * 1024 * 1024)),
    }?;

    let decompressed = common::decode_weakaura(&src, max_size).or_else(|e| {
        let e = cx.string(e);
        cx.throw(e)
    })?;
    let decompressed = String::from_utf8_lossy(&decompressed);

    Deserializer::from_str(&decompressed)
        .deserialize_first(&mut cx)
        .or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let value = cx.argument::<JsValue>(0)?;

    Serializer::serialize(&mut cx, value)
        .and_then(|serialized| common::encode_weakaura(&serialized))
        .map(|result| cx.string(result))
        .or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })
}
