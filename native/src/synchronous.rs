use neon::prelude::*;

use super::common;

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let src = cx.argument::<JsString>(0)?.value();

    common::decode_weakaura(&src)
        .map(|json| cx.string(json))
        .or_else(|e| cx.throw_error(e))
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsString> {
    let json = cx.argument::<JsString>(0)?.value();

    common::encode_weakaura(&json)
        .map(|result| cx.string(result))
        .or_else(|e| cx.throw_error(e))
}
