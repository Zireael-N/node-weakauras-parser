use neon::prelude::*;

use super::ace_serialize::{Deserializer as LegacyDeserializer, Serializer};
use super::common::{self, StringVersion};
use super::lib_serialize::Deserializer;

struct DecodeTask(String, Option<usize>);
impl Task for DecodeTask {
    type Output = (Vec<u8>, StringVersion);
    type Error = &'static str;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        common::decode_weakaura(&self.0, self.1)
    }

    fn complete(self, mut cx: TaskContext, result: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
        let (decompressed, version) = result.or_else(|e| {
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
