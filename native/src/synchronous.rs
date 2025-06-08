use neon::prelude::*;

use super::common;
use weakauras_codec::{decode, encode};

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let src = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_size = common::parse_max_size(cx.argument_opt(1), &mut cx)?;

    decode(src.as_bytes().trim_ascii_end(), max_size)
        .or_else(|e| cx.throw_error(e.to_string()))
        .and_then(|v| serde_json::to_string(&v).or_else(|e| cx.throw_error(e.to_string())))
        .map(|json| cx.string(json))
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);
    let src = serde_json::from_str(&json).or_else(|e| cx.throw_error(e.to_string()))?;
    let string_version = common::parse_string_version(cx.argument_opt(1), &mut cx)?;

    encode(&src, string_version)
        .map(|result| cx.string(result))
        .or_else(|e| cx.throw_error(e.to_string()))
}
