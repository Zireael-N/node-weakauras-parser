use neon::prelude::*;

use super::ace_serialize::{Deserializer as LegacyDeserializer, Serializer};
use super::common::{self, StringVersion};
use super::lib_serialize::Deserializer;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsValue> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = match cx.argument_opt(1) {
        Some(v) => common::transform_max_size(v, &mut cx),
        None => Ok(Some(8 * 1024 * 1024)),
    }?;

    let (decompressed, version) = common::decode_weakaura(&src, max_size).or_else(|e| {
        let e = cx.string(e);
        cx.throw(e)
    })?;

    let result = if let StringVersion::BinarySerialization = version {
        Deserializer::from_slice(&decompressed).deserialize_first(&mut cx)
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decompressed)).deserialize_first(&mut cx)
    };

    result.or_else(|e| {
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
