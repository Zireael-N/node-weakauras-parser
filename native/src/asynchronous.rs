use neon::prelude::*;

use super::common;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let src = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_size = common::parse_max_size(cx.argument_opt(1), &mut cx)?;

    let promise = cx
        .task(move || common::decode_weakaura(&src, max_size))
        .promise(|mut cx, result| {
            result
                .map(|json| cx.string(json))
                .or_else(|e| cx.throw_error(e))
        });

    Ok(promise)
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let src = cx.argument::<JsString>(0)?.value(&mut cx);
    let string_version = common::parse_string_version(cx.argument_opt(1), &mut cx)?;

    let promise = cx
        .task(move || common::encode_weakaura(&src, string_version))
        .promise(|mut cx, result| {
            result
                .map(|serialized| cx.string(serialized))
                .or_else(|e| cx.throw_error(e))
        });

    Ok(promise)
}
