use neon::prelude::*;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
fn f64_to_parts(v: f64) -> (u64, i16, i8) {
    let bits = v.to_bits();
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

pub struct Serializer<'c, 'v, C: Context<'v> + 'c> {
    cx: &'c mut C,
    remaining_depth: usize,
    scratch: String,
    marker: std::marker::PhantomData<&'v u8>,
}

impl<'c, 'v, C> Serializer<'c, 'v, C>
where
    C: Context<'v> + 'c,
{
    pub fn serialize(cx: &'c mut C, value: Handle<'v, JsValue>) -> Result<String, &'static str> {
        let mut serializer = Self {
            cx,
            remaining_depth: 128,
            scratch: String::new(),
            marker: std::marker::PhantomData,
        };

        let mut result = String::with_capacity(4);
        result.push_str("^1");
        result.push_str(&serializer.serialize_helper(value)?);
        result.push_str("^^");
        Ok(result)
    }

    fn serialize_helper(&mut self, value: Handle<'v, JsValue>) -> Result<String, &'static str> {
        // Taken from serde_json
        macro_rules! check_recursion {
            ($($body:tt)*) => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err("recursion limit exceeded");
                }

                $($body)*

                self.remaining_depth += 1;
            }
        }

        if value.is_a::<JsNull>() || value.is_a::<JsUndefined>() {
            Ok("^Z".into())
        } else if let Ok(val) = value.downcast::<JsBoolean>() {
            Ok((if val.value() { "^B" } else { "^b" }).into())
        } else if let Ok(val) = value.downcast::<JsString>() {
            Ok(format!("^S{}", self.serialize_string(&val.value())))
        } else if let Ok(val) = value.downcast::<JsNumber>() {
            Ok(Self::serialize_number(val.value()))
        } else if let Ok(val) = value.downcast::<JsArray>() {
            let len = val.len();
            let mut result = String::with_capacity(len as usize * 6 + 4);
            result.push_str("^T");
            for i in 0..len {
                let v = val.get(self.cx, i).unwrap();
                result.push_str(&format!("^N{}", i + 1));
                check_recursion! {
                    result.push_str(&self.serialize_helper(v)?);
                }
            }
            result.push_str("^t");
            Ok(result)
        } else if let Ok(val) = value.downcast::<JsObject>() {
            let properties = val.get_own_property_names(self.cx).unwrap();
            let len = properties.len();
            let mut result = String::with_capacity(len as usize * 6 + 4);
            result.push_str("^T");
            for i in 0..len {
                let name = properties.get(self.cx, i).unwrap();
                let value = val.get(self.cx, name).unwrap();
                check_recursion! {
                    result.push_str(&format!(
                        "{}{}",
                        self.serialize_helper(name)?,
                        self.serialize_helper(value)?,
                    ));
                }
            }
            result.push_str("^t");
            Ok(result)
        } else {
            Err("unsupported type")
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::float_cmp))]
    fn serialize_number(value: f64) -> String {
        if value.is_nan() {
            "^N1.#IND".into()
        } else if !value.is_finite() {
            format!("^N{}", if value > 0.0 { "1.#INF" } else { "-1.#INF" })
        } else if value.to_string().parse::<f64>().unwrap() == value {
            format!("^N{}", value)
        } else {
            let (mantissa, exponent, sign) = f64_to_parts(value);
            format!("^F{}{}^f{}", if sign < 0 { "-" } else { "" }, mantissa, exponent)
        }
    }

    fn serialize_string<'a>(&'a mut self, value: &'a str) -> &'a str {
        self.scratch.clear();

        let mut copy_from = 0;
        for (i, byte) in value.bytes().enumerate() {
            let replacement = match byte {
                v @ 0x00..=0x1D | v @ 0x1F..=0x20 => v + 64,
                0x1E => 0x7A,
                0x5E => 0x7D,
                0x7E => 0x7C,
                0x7F => 0x7B,
                _ => continue,
            };

            if self.scratch.capacity() == 0 {
                self.scratch.reserve(value.len() + 1);
            }

            self.scratch.push_str(&value[copy_from..i]);
            self.scratch.push_str(&format!("~{}", replacement as char));
            copy_from = i + 1;
        }

        if self.scratch.is_empty() {
            value
        } else {
            self.scratch.push_str(&value[copy_from..]);
            &self.scratch[..]
        }
    }
}
