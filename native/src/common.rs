use neon::prelude::*;
use weakauras_codec::OutputStringVersion;

pub fn parse_max_size<'a>(
    v: Option<Handle<'a, JsValue>>,
    cx: &'a mut FunctionContext,
) -> NeonResult<Option<usize>> {
    match v {
        Some(v) => {
            if v.is_a::<JsUndefined, _>(cx) {
                Ok(Some(8 * 1024 * 1024))
            } else {
                v.downcast_or_throw::<JsNumber, _>(cx).and_then(|v| {
                    let v = v.value(cx);
                    if v == f64::INFINITY {
                        Ok(Some(usize::MAX))
                    } else if v.is_finite() && v >= 0.0 {
                        Ok(Some(v.trunc() as usize))
                    } else {
                        cx.throw_type_error(
                            "Invalid value, expected a positive finite number or +Infinity",
                        )
                    }
                })
            }
        }
        None => Ok(Some(8 * 1024 * 1024)),
    }
}

pub fn parse_string_version<'a>(
    v: Option<Handle<'a, JsValue>>,
    cx: &'a mut FunctionContext,
) -> NeonResult<OutputStringVersion> {
    match v {
        Some(v) => {
            if v.is_a::<JsUndefined, _>(cx) {
                Ok(OutputStringVersion::BinarySerialization)
            } else {
                v.downcast_or_throw::<JsNumber, _>(cx).and_then(|v| {
                    let v = v.value(cx).trunc() as u64;
                    match v {
                        1 => Ok(OutputStringVersion::Deflate),
                        2 => Ok(OutputStringVersion::BinarySerialization),
                        _ => cx.throw_type_error("Invalid value"),
                    }
                })
            }
        }
        None => Ok(OutputStringVersion::BinarySerialization),
    }
}
