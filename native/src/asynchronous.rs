use neon::prelude::*;

use super::common;
use weakauras_codec::{decode, encode};

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let src = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_size = common::parse_max_size(cx.argument_opt(1), &mut cx)?;

    let promise = cx
        .task(move || decode(src.as_bytes().trim_ascii_end(), max_size))
        .promise(|mut cx, result| {
            result
                .or_else(|e| cx.throw_error(e.to_string()))
                .and_then(|v| serde_json::to_string(&v).or_else(|e| cx.throw_error(e.to_string())))
                .map(|json| cx.string(json))
        });

    Ok(promise)
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);
    let src = serde_json::from_str(&json).or_else(|e| cx.throw_error(e.to_string()))?;
    let string_version = common::parse_string_version(cx.argument_opt(1), &mut cx)?;

    let promise = cx
        .task(move || encode(&src, string_version))
        .promise(|mut cx, result| {
            result
                .map(|serialized| cx.string(serialized))
                .or_else(|e| cx.throw_error(e.to_string()))
        });

    Ok(promise)
}
