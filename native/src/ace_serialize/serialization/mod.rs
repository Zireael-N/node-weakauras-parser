use crate::macros::check_recursion;
use serde_json::Value;

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

fn write_integer_to_a_string<I>(string: &mut String, value: I)
where
    I: itoa::Integer,
{
    let mut buffer = itoa::Buffer::new();
    let serialized = buffer.format(value);
    string.push_str(serialized)
}

pub struct Serializer {
    remaining_depth: usize,
    result: String,
}

impl Serializer {
    pub fn serialize(
        value: &Value,
        approximate_len: Option<usize>,
    ) -> Result<String, &'static str> {
        let mut serializer = Self {
            remaining_depth: 128,
            result: String::with_capacity(approximate_len.unwrap_or(1024)),
        };

        serializer.result.push_str("^1");
        serializer.serialize_helper(value)?;
        serializer.result.push_str("^^");

        Ok(serializer.result)
    }

    fn serialize_helper(&mut self, value: &Value) -> Result<(), &'static str> {
        match *value {
            Value::Null => self.result.push_str("^Z"),
            Value::Bool(b) => self.result.push_str(if b { "^B" } else { "^b" }),
            Value::String(ref s) => {
                self.result.push_str("^S");
                self.serialize_string(s)
            }
            Value::Number(ref n) => n
                .as_f64()
                .ok_or("Failed to parse a number")
                .and_then(|n| self.serialize_number(n))?,
            Value::Array(ref vec) => {
                self.result.reserve(vec.len() * 6 + 4);

                self.result.push_str("^T");
                for (i, v) in vec.iter().enumerate() {
                    check_recursion!(self, {
                        self.result.push_str("^N");
                        write_integer_to_a_string(&mut self.result, i + 1);
                        self.serialize_helper(v)?;
                    });
                }
                self.result.push_str("^t");
            }
            Value::Object(ref m) => {
                self.result.reserve(m.len() * 6 + 4);

                self.result.push_str("^T");
                for (key, value) in m.iter() {
                    check_recursion!(self, {
                        self.result.push_str(if key.parse::<i32>().is_ok() {
                            "^N"
                        } else {
                            "^S"
                        });
                        self.result.push_str(key);
                        self.serialize_helper(value)?;
                    });
                }
                self.result.push_str("^t");
            }
        }

        Ok(())
    }

    fn serialize_number(&mut self, value: f64) -> Result<(), &'static str> {
        if value.is_nan() {
            return Err("AceSerializer does not support NaNs");
        } else if !value.is_finite() {
            self.result.push_str("^N");
            self.result
                .push_str(if value > 0.0 { "1.#INF" } else { "-1.#INF" })
        } else {
            let mut buffer = ryu::Buffer::new();
            let str_value = buffer.format_finite(value);

            if str_value.parse::<f64>().unwrap() == value {
                self.result.reserve(str_value.len() + 2);
                self.result.push_str("^N");
                self.result.push_str(str_value);
            } else {
                let (mantissa, exponent, sign) = f64_to_parts(value);
                self.result.push_str("^F");
                if sign < 0 {
                    self.result.push('-');
                }
                write_integer_to_a_string(&mut self.result, mantissa);
                self.result.push_str("^f");
                write_integer_to_a_string(&mut self.result, exponent);
            }
        }

        Ok(())
    }

    fn serialize_string(&mut self, value: &str) {
        self.result.reserve(value.len());

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

            self.result.push_str(&value[copy_from..i]);
            self.result.push('~');
            self.result.push(replacement as char);
            copy_from = i + 1;
        }

        self.result.push_str(&value[copy_from..]);
    }
}
