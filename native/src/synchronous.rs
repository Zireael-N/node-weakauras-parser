use neon::prelude::*;

use super::common;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = match cx.argument_opt(1) {
        Some(v) => common::transform_max_size(v, &mut cx),
        None => Ok(Some(8 * 1024 * 1024)),
    }?;

    common::decode_weakaura(&src, max_size)
        .map(|json| cx.string(json))
        .or_else(|e| cx.throw_error(e))
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value();

    common::encode_weakaura(&json)
        .map(|result| cx.string(result))
        .or_else(|e| cx.throw_error(e))
}
