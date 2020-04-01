use neon::prelude::*;

use super::common;
use super::deserialization::Deserializer;
use super::serialization::Serializer;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsValue> {
    let src = cx.argument::<JsString>(0)?.value();

    let decompressed = common::decode_weakaura(&src).or_else(|e| cx.throw_error(e))?;
    let decompressed = String::from_utf8_lossy(&decompressed);

    Deserializer::from_str(&decompressed)
        .deserialize_first(&mut cx)
        .or_else(|e| cx.throw_error(e))
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let value = cx.argument::<JsValue>(0)?;

    Serializer::serialize(&mut cx, value)
        .and_then(|serialized| common::encode_weakaura(&serialized))
        .map(|result| cx.string(result))
        .or_else(|e| cx.throw_error(e))
}
