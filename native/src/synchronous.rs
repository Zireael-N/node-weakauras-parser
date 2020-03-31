use neon::prelude::*;

use super::common;
use super::deserialization::Deserializer;
use super::serialization::Serializer;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsValue> {
    let src = cx.argument::<JsString>(0)?.value();

    let decompressed = common::decode_weakaura(&src).unwrap();
    let decompressed = String::from_utf8_lossy(&decompressed);

    let deserialized = Deserializer::from_str(&decompressed)
        .deserialize_first(&mut cx)
        .unwrap();

    Ok(deserialized)
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let value = cx.argument::<JsValue>(0)?;

    let serialized = Serializer::serialize(&mut cx, value).unwrap();
    let result = common::encode_weakaura(&serialized).unwrap();

    Ok(cx.string(result))
}
