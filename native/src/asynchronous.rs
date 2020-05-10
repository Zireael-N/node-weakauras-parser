use neon::prelude::*;

use super::common;
use super::deserialization::Deserializer;
use super::serialization::Serializer;

use std::borrow::Cow;

struct DecodeTask(String, Option<usize>);
impl Task for DecodeTask {
    type Output = String;
    type Error = &'static str;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let decompressed = common::decode_weakaura(&self.0, self.1)?;

        // Avoid an unnecessary copy that would occur with Cow::into_owned():
        Ok(match String::from_utf8_lossy(&decompressed) {
            Cow::Owned(v) => v,
            Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(decompressed) },
        })
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        let result = result.or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })?;

        Deserializer::from_str(&result).deserialize_first(&mut cx).or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })
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
        let result = result.or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })?;

        Ok(cx.string(result))
    }
}

pub fn decode_weakaura(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let src = cx.argument::<JsString>(0)?.value();
    let max_size = cx
        .argument::<JsValue>(1)
        .and_then(|v| common::transform_max_size(v, &mut cx))?;
    let cb = cx.argument::<JsFunction>(2)?;

    DecodeTask(src, max_size).schedule(cb);

    Ok(cx.undefined())
}

pub fn encode_weakaura(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let value = cx.argument::<JsValue>(0)?;
    let cb = cx.argument::<JsFunction>(1)?;

    Serializer::serialize(&mut cx, value)
        .map(|serialized| {
            EncodeTask(serialized).schedule(cb);
            cx.undefined()
        })
        .or_else(|e| {
            let e = cx.string(e);
            cx.throw(e)
        })
}
