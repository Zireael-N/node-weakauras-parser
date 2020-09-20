mod reader;

use super::{EmbeddedTypeTag, TypeTag, MINOR};
use reader::SliceReader;
use serde_json::{map::Map, Number, Value};

fn f64_to_json_value(value: f64) -> Value {
    match Number::from_f64(value) {
        Some(num) => Value::Number(num),
        None => Value::Null,
    }
}

pub struct Deserializer<'s> {
    remaining_depth: usize,
    reader: SliceReader<'s>,

    table_refs: Vec<Value>,
    string_refs: Vec<String>,
}

impl<'s> Deserializer<'s> {
    pub fn from_slice(v: &'s [u8]) -> Self {
        Self {
            remaining_depth: 128,
            reader: SliceReader::new(v),

            table_refs: Vec::new(),
            string_refs: Vec::new(),
        }
    }

    /// Returns an array of deserialized values
    #[allow(dead_code)]
    pub fn deserialize(mut self) -> Result<Vec<Value>, &'static str> {
        match self.reader.read_u8() {
            Some(MINOR) => (),
            _ => return Err("Invalid serialized data"),
        }

        let mut result = Vec::new();

        while let Some(v) = self.deserialize_helper()? {
            result.push(v);
        }

        Ok(result)
    }

    /// Returns the first deserialized value
    #[allow(dead_code)]
    pub fn deserialize_first(mut self) -> Result<Value, &'static str> {
        match self.reader.read_u8() {
            Some(MINOR) => (),
            _ => return Err("Invalid serialized data"),
        }

        self.deserialize_helper().map(|result| result.unwrap_or(Value::Null))
    }

    fn deserialize_helper(&mut self) -> Result<Option<Value>, &'static str> {
        match self.reader.read_u8() {
            None => Ok(None),
            Some(value) => {
                if value & 1 == 1 {
                    // `NNNN NNN1`: a 7 bit non-negative int
                    Ok(Some(f64_to_json_value((value >> 1) as f64)))
                } else if value & 3 == 2 {
                    // * `CCCC TT10`: a 2 bit type index and 4 bit count (strlen, #tab, etc.)
                    //     * Followed by the type-dependent payload
                    let tag = EmbeddedTypeTag::from_u8((value & 0x0F) >> 2).ok_or("Invalid embedded tag")?;
                    let len = value >> 4;

                    self.deserialize_embedded(tag, len).map(Option::Some)
                } else if value & 7 == 4 {
                    // * `NNNN S100`: the lower four bits of a 12 bit int and 1 bit for its sign
                    //     * Followed by a byte for the upper bits
                    let next_byte = self.reader.read_u8().ok_or("Unexpected EOF")? as u16;
                    let packed = (next_byte << 8) + value as u16;

                    let value = if value & 15 == 12 {
                        -((packed >> 4) as f64)
                    } else {
                        (packed >> 4) as f64
                    };

                    Ok(Some(f64_to_json_value(value)))
                } else {
                    // * `TTTT T000`: a 5 bit type index
                    //     * Followed by the type-dependent payload, including count(s) if needed
                    let tag = TypeTag::from_u8(value >> 3).ok_or("Invalid tag")?;

                    self.deserialize_one(tag).map(Option::Some)
                }
            }
        }
    }

    fn extract_key(&mut self) -> Result<String, &'static str> {
        match self.deserialize_helper() {
            Ok(Some(Value::String(s))) => Ok(s),
            Ok(Some(Value::Number(n))) => n
                .as_f64()
                .map(|n| n.to_string())
                .ok_or("Deserialization error: failed to parse a floating-point number"),
            Ok(Some(Value::Bool(b))) => Ok((if b { "true" } else { "false" }).into()),
            Ok(Some(_)) => Err("Deserialization error: Unsupported type for an object key"),
            Ok(None) => Err("Unexpected EOF"),
            Err(e) => Err(e),
        }
    }

    #[inline(always)]
    fn extract_value(&mut self) -> Result<Value, &'static str> {
        match self.deserialize_helper() {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err("Unexpected EOF"),
            Err(e) => Err(e),
        }
    }

    fn deserialize_embedded(&mut self, tag: EmbeddedTypeTag, len: u8) -> Result<Value, &'static str> {
        match tag {
            EmbeddedTypeTag::Str => self.deserialize_string(len as usize),
            EmbeddedTypeTag::Map => self.deserialize_map(len as usize),
            EmbeddedTypeTag::Array => self.deserialize_array(len as usize),
            // For MIXED, the 4-bit count contains two 2-bit counts that are one less than the true count.
            EmbeddedTypeTag::Mixed => self.deserialize_mixed(((len & 3) + 1) as usize, ((len >> 2) + 1) as usize),
        }
    }

    fn deserialize_one(&mut self, tag: TypeTag) -> Result<Value, &'static str> {
        match tag {
            TypeTag::Null => Ok(Value::Null),

            TypeTag::Int16Pos => self.deserialize_int(2).map(|v| f64_to_json_value(v as f64)),
            TypeTag::Int16Neg => self.deserialize_int(2).map(|v| f64_to_json_value(-(v as f64))),
            TypeTag::Int24Pos => self.deserialize_int(3).map(|v| f64_to_json_value(v as f64)),
            TypeTag::Int24Neg => self.deserialize_int(3).map(|v| f64_to_json_value(-(v as f64))),
            TypeTag::Int32Pos => self.deserialize_int(4).map(|v| f64_to_json_value(v as f64)),
            TypeTag::Int32Neg => self.deserialize_int(4).map(|v| f64_to_json_value(-(v as f64))),
            TypeTag::Int64Pos => self.deserialize_int(7).map(|v| f64_to_json_value(v as f64)),
            TypeTag::Int64Neg => self.deserialize_int(7).map(|v| f64_to_json_value(-(v as f64))),

            TypeTag::Float => self.deserialize_f64().map(f64_to_json_value),
            TypeTag::FloatStrPos => self.deserialize_f64_from_str().map(f64_to_json_value),
            TypeTag::FloatStrNeg => self.deserialize_f64_from_str().map(|v| f64_to_json_value(-v)),

            TypeTag::True => Ok(Value::Bool(true)),
            TypeTag::False => Ok(Value::Bool(false)),

            TypeTag::Str8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_string(len as usize)
            }
            TypeTag::Str16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_string(len as usize)
            }
            TypeTag::Str24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_string(len as usize)
            }

            TypeTag::Map8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_map(len as usize)
            }
            TypeTag::Map16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_map(len as usize)
            }
            TypeTag::Map24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_map(len as usize)
            }

            TypeTag::Array8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_array(len as usize)
            }
            TypeTag::Array16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_array(len as usize)
            }
            TypeTag::Array24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_array(len as usize)
            }

            TypeTag::Mixed8 => {
                let array_len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                let map_len = self.reader.read_u8().ok_or("Unexpected EOF")?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }
            TypeTag::Mixed16 => {
                let array_len = self.deserialize_int(2)?;
                let map_len = self.deserialize_int(2)?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }
            TypeTag::Mixed24 => {
                let array_len = self.deserialize_int(3)?;
                let map_len = self.deserialize_int(3)?;

                self.deserialize_mixed(array_len as usize, map_len as usize)
            }

            TypeTag::StrRef8 => {
                let index = self.reader.read_u8().ok_or("Unexpected EOF")? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(Value::String(s.clone())),
                }
            }
            TypeTag::StrRef16 => {
                let index = self.deserialize_int(2)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(Value::String(s.clone())),
                }
            }
            TypeTag::StrRef24 => {
                let index = self.deserialize_int(3)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(Value::String(s.clone())),
                }
            }

            TypeTag::MapRef8 => {
                let index = self.reader.read_u8().ok_or("Unexpected EOF")? - 1;
                match self.table_refs.get(index as usize) {
                    None => Err("Invalid table reference"),
                    Some(v) => Ok(v.clone()),
                }
            }
            TypeTag::MapRef16 => {
                let index = self.deserialize_int(2)? - 1;
                match self.table_refs.get(index as usize) {
                    None => Err("Invalid table reference"),
                    Some(v) => Ok(v.clone()),
                }
            }
            TypeTag::MapRef24 => {
                let index = self.deserialize_int(3)? - 1;
                match self.table_refs.get(index as usize) {
                    None => Err("Invalid table reference"),
                    Some(v) => Ok(v.clone()),
                }
            }
        }
    }

    fn deserialize_string(&mut self, len: usize) -> Result<Value, &'static str> {
        match self.reader.read_string(len) {
            None => Err("Unexpected EOF"),
            Some(s) => {
                let s = s.into_owned();
                if len > 2 {
                    self.string_refs.push(s.clone());
                }

                Ok(Value::String(s))
            }
        }
    }

    fn deserialize_f64(&mut self) -> Result<f64, &'static str> {
        match self.reader.read_f64() {
            None => Err("Unexpected EOF"),
            Some(v) => Ok(v),
        }
    }

    fn deserialize_f64_from_str(&mut self) -> Result<f64, &'static str> {
        let len = self.reader.read_u8().ok_or("Unexpected EOF")?;

        match self.reader.read_bytes(len as usize) {
            None => Err("Unexpected EOF"),
            Some(bytes) => std::str::from_utf8(bytes)
                .ok()
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or("Cannot parse a number"),
        }
    }

    fn deserialize_int(&mut self, bytes: usize) -> Result<u64, &'static str> {
        match self.reader.read_int(bytes) {
            None => Err("Unexpected EOF"),
            Some(v) => Ok(v),
        }
    }

    fn deserialize_map(&mut self, len: usize) -> Result<Value, &'static str> {
        let mut map = Map::new();

        // Taken from serde_json
        macro_rules! check_recursion {
            ($($body:tt)*) => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err("Recursion limit exceeded");
                }

                $($body)*

                self.remaining_depth += 1;
            }
        }

        for _ in 0..len {
            check_recursion! {
                let (key, value) = (self.extract_key()?, self.extract_value()?);

                map.insert(key, value);
            }
        }

        let map = Value::Object(map);
        self.table_refs.push(map.clone());
        Ok(map)
    }

    fn deserialize_array(&mut self, len: usize) -> Result<Value, &'static str> {
        let mut v = Vec::new();

        // Taken from serde_json
        macro_rules! check_recursion {
            ($($body:tt)*) => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err("Recursion limit exceeded");
                }

                $($body)*

                self.remaining_depth += 1;
            }
        }

        for _ in 0..len {
            check_recursion! {
                v.push(self.extract_value()?);
            }
        }

        let v = Value::Array(v);
        self.table_refs.push(v.clone());
        Ok(v)
    }

    fn deserialize_mixed(&mut self, array_len: usize, map_len: usize) -> Result<Value, &'static str> {
        let mut map = Map::new();

        // Taken from serde_json
        macro_rules! check_recursion {
            ($($body:tt)*) => {
                self.remaining_depth -= 1;
                if self.remaining_depth == 0 {
                    return Err("Recursion limit exceeded");
                }

                $($body)*

                self.remaining_depth += 1;
            }
        }

        for i in 1..=array_len {
            check_recursion! {
                let el = self.extract_value()?;

                map.insert(i.to_string(), el);
            }
        }

        for _ in 0..map_len {
            check_recursion! {
                let (key, value) = (self.extract_key()?, self.extract_value()?);

                map.insert(key, value);
            }
        }

        let map = Value::Object(map);
        self.table_refs.push(map.clone());
        Ok(map)
    }
}
