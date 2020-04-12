use serde_json::{map::Map, Number, Value};

mod reader;
use reader::StrReader;

pub struct Deserializer<'s> {
    remaining_depth: usize,
    reader: StrReader<'s>,
}

impl<'s> Deserializer<'s> {
    pub fn from_str(slice: &'s str) -> Self {
        Self {
            remaining_depth: 128,
            reader: StrReader::new(slice),
        }
    }

    /// Returns an array of deserialized values
    #[allow(dead_code)]
    pub fn deserialize(mut self) -> Result<Value, &'static str> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        let mut result = Vec::new();

        while self.reader.peek_identifier().is_ok() {
            if let Some(v) = self.deserialize_helper()? {
                result.push(v);
            }
        }

        Ok(Value::Array(result))
    }

    /// Returns the first deserialized value
    #[allow(dead_code)]
    pub fn deserialize_first(mut self) -> Result<Value, &'static str> {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        let value = match self.deserialize_helper()? {
            Some(v) => v,
            _ => Value::Null,
        };

        Ok(value)
    }

    fn deserialize_helper(&mut self) -> Result<Option<Value>, &'static str> {
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

        Ok(Some(match self.reader.read_identifier()? {
            "^^" => return Ok(None),
            "^Z" => Value::Null,
            "^B" => Value::Bool(true),
            "^b" => Value::Bool(false),
            "^S" => Value::String(self.reader.parse_str()?.to_string()),
            "^N" => {
                self.reader
                    .read_until_next()
                    .and_then(Self::deserialize_number)
                    .map(|v| match Number::from_f64(v) {
                        Some(v) => Value::Number(v),
                        None => Value::Null,
                    })?
            }
            "^F" => {
                let mantissa = self
                    .reader
                    .read_until_next()
                    .and_then(|v| v.parse::<f64>().map_err(|_| "failed to parse a number"))?;
                let exponent = match self.reader.read_identifier()? {
                    "^f" => self
                        .reader
                        .read_until_next()
                        .and_then(|v| v.parse::<f64>().map_err(|_| "failed to parse a number"))?,
                    _ => return Err("missing exponent"),
                };

                match Number::from_f64(mantissa * 2f64.powf(exponent)) {
                    Some(v) => Value::Number(v),
                    None => Value::Null,
                }
            }
            "^T" => {
                let mut result = Map::new();
                loop {
                    match self.reader.peek_identifier()? {
                        "^t" => {
                            let _ = self.reader.read_identifier();
                            break;
                        }
                        _ => {
                            check_recursion! {
                                let key = self.deserialize_helper()?.ok_or("missing key").and_then(|key| match key {
                                    Value::String(s) => Ok(s),
                                    Value::Number(n) => n.as_f64().map(|v| v.to_string()).ok_or("failed to parse a number"),
                                    Value::Bool(b) => Ok((if b { "true" } else { "false" }).into()),
                                    _ => Err("unsupported key type for a map"),
                                })?;

                                let value = match self.reader.peek_identifier()? {
                                    "^t" => return Err("unexpected end of a table"),
                                    _ => self.deserialize_helper()?.ok_or("missing value")?,
                                };

                                result.insert(key, value);
                            }
                        }
                    }
                }
                Value::Object(result)
            }
            _ => return Err("invalid identifier"),
        }))
    }

    fn deserialize_number(data: &str) -> Result<f64, &'static str> {
        match data {
            "1.#INF" | "inf" => Ok(std::f64::INFINITY),
            "-1.#INF" | "-inf" => Ok(std::f64::NEG_INFINITY),
            v => v.parse().map_err(|_| "failed to parse a number"),
        }
    }
}
