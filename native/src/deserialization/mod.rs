use neon::prelude::*;

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
    pub fn deserialize<'c, 'v, C: 'c>(mut self, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        let mut index = 0;
        let result = cx.empty_array();

        while self.reader.peek_identifier().is_ok() {
            if let Some(v) = self.deserialize_helper(cx)? {
                result.set(cx, index, v).map_err(|_| "failed to set property")?;
                index += 1;
            }
        }

        Ok(result.as_value(cx))
    }

    /// Returns the first deserialized value
    #[allow(dead_code)]
    pub fn deserialize_first<'c, 'v, C: 'c>(mut self, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        self.reader.read_identifier().and_then(|v| {
            if v == "^1" {
                Ok(())
            } else {
                Err("supplied data is not AceSerializer data (rev 1)")
            }
        })?;

        let value = match self.deserialize_helper(cx)? {
            Some(v) => v,
            _ => cx.undefined().as_value(cx),
        };

        Ok(value)
    }

    fn deserialize_helper<'c, 'v, C: 'c>(&mut self, cx: &'c mut C) -> Result<Option<Handle<'v, JsValue>>, &'static str>
    where
        C: Context<'v>,
    {
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
            "^Z" => cx.null().as_value(cx),
            "^B" => cx.boolean(true).as_value(cx),
            "^b" => cx.boolean(false).as_value(cx),
            "^S" => cx.string(self.reader.parse_str()?).as_value(cx),
            "^N" => cx
                .number(self.reader.read_until_next().and_then(Self::deserialize_number)?)
                .as_value(cx),
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

                cx.number(mantissa * (2f64.powf(exponent))).as_value(cx)
            }
            "^T" => {
                let result = JsObject::new(cx);
                loop {
                    match self.reader.peek_identifier()? {
                        "^t" => {
                            let _ = self.reader.read_identifier();
                            break;
                        }
                        _ => {
                            check_recursion! {
                                let key = self.deserialize_helper(cx)?.ok_or("missing key")?;
                                let value = match self.reader.peek_identifier()? {
                                    "^t" => return Err("unexpected end of a table"),
                                    _ => self.deserialize_helper(cx)?.ok_or("missing value")?,
                                };
                                result.set(cx, key, value).map_err(|_| "failed to set property")?;
                            }
                        }
                    }
                }
                result.as_value(cx)
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
