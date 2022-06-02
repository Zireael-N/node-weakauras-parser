use neon::prelude::*;

use super::common;

struct DecodeTask(String, Option<usize>);
impl Task for DecodeTask {
    type Output = String;
    type Error = &'static str;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        common::decode_weakaura(&self.0, self.1)
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        result.map(|json| cx.string(json)).or_else(|e| cx.throw_error(e))
    }
}

struct EncodeTask(String);
impl Task for EncodeTask {
    type Output = String;
    type Error = &'static str;
    type JsEvent = JsString;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        common::encode_weakaura(&self.0)
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        result
            .map(|serialized| cx.string(serialized))
            .or_else(|e| cx.throw_error(e))
    }
}

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = common::parse_max_size(cx.argument_opt(1), &mut cx)?;
    let cb = cx.argument::<JsFunction>(2)?;

    DecodeTask(src, max_size).schedule(cb);

    Ok(cx.undefined())
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let src = cx.argument::<JsString>(0)?.value();
    let cb = cx.argument::<JsFunction>(1)?;

    EncodeTask(src).schedule(cb);

    Ok(cx.undefined())
}
