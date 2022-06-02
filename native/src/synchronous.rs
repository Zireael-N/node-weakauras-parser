use neon::prelude::*;

use super::common;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let src = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_size = common::parse_max_size(cx.argument_opt(1), &mut cx)?;

    common::decode_weakaura(&src, max_size)
        .map(|json| cx.string(json))
        .or_else(|e| cx.throw_error(e))
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value(&mut cx);

    common::encode_weakaura(&json)
        .map(|result| cx.string(result))
        .or_else(|e| cx.throw_error(e))
}
