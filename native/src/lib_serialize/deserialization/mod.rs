use neon::prelude::*;

mod reader;

use super::{EmbeddedTypeTag, TypeTag, MINOR};
use reader::SliceReader;

pub struct Deserializer<'s, 'v> {
    remaining_depth: usize,
    reader: SliceReader<'s>,

    table_refs: Vec<Handle<'v, JsValue>>,
    string_refs: Vec<String>,
}

impl<'s, 'v> Deserializer<'s, 'v> {
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
    pub fn deserialize<'c, C: 'c>(mut self, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match self.reader.read_u8() {
            Some(MINOR) => (),
            _ => return Err("Invalid serialized data"),
        }

        let mut index = 0;
        let result = cx.empty_array();

        while let Some(v) = self.deserialize_helper(cx)? {
            result.set(cx, index, v).unwrap();
            index += 1;
        }

        Ok(result.as_value(cx))
    }

    /// Returns the first deserialized value
    #[allow(dead_code)]
    pub fn deserialize_first<'c, C: 'c>(mut self, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match self.reader.read_u8() {
            Some(MINOR) => (),
            _ => return Err("Invalid serialized data"),
        }

        self.deserialize_helper(cx).map(|result| result.unwrap_or_else(|| cx.undefined().as_value(cx)))
    }

    fn deserialize_helper<'c, C: 'c>(&mut self, cx: &'c mut C) -> Result<Option<Handle<'v, JsValue>>, &'static str>
    where
        C: Context<'v>,
    {
        match self.reader.read_u8() {
            None => Ok(None),
            Some(value) => {
                if value & 1 == 1 {
                    // `NNNN NNN1`: a 7 bit non-negative int
                    Ok(Some(cx.number((value >> 1) as f64).as_value(cx)))
                } else if value & 3 == 2 {
                    // * `CCCC TT10`: a 2 bit type index and 4 bit count (strlen, #tab, etc.)
                    //     * Followed by the type-dependent payload
                    let tag = EmbeddedTypeTag::from_u8((value & 0x0F) >> 2).ok_or("Invalid embedded tag")?;
                    let len = value >> 4;

                    self.deserialize_embedded(tag, len, cx).map(Option::Some)
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

                    Ok(Some(cx.number(value).as_value(cx)))
                } else {
                    // * `TTTT T000`: a 5 bit type index
                    //     * Followed by the type-dependent payload, including count(s) if needed
                    let tag = TypeTag::from_u8(value >> 3).ok_or("Invalid tag")?;

                    self.deserialize_one(tag, cx).map(Option::Some)
                }
            }
        }
    }

    #[inline(always)]
    fn extract_value<'c, C: 'c>(&mut self, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match self.deserialize_helper(cx) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err("Unexpected EOF"),
            Err(e) => Err(e),
        }
    }

    fn deserialize_embedded<'c, C: 'c>(&mut self, tag: EmbeddedTypeTag, len: u8, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match tag {
            EmbeddedTypeTag::Str => self.deserialize_string(len as usize, cx),
            EmbeddedTypeTag::Map => self.deserialize_map(len as usize, cx),
            EmbeddedTypeTag::Array => self.deserialize_array(len as usize, cx),
            // For MIXED, the 4-bit count contains two 2-bit counts that are one less than the true count.
            EmbeddedTypeTag::Mixed => self.deserialize_mixed(((len & 3) + 1) as usize, ((len >> 2) + 1) as usize, cx),
        }
    }

    fn deserialize_one<'c, C: 'c>(&mut self, tag: TypeTag, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match tag {
            TypeTag::Null => Ok(cx.null().as_value(cx)),

            TypeTag::Int16Pos => self.deserialize_int(2).map(|v| cx.number(v as f64).as_value(cx)),
            TypeTag::Int16Neg => self.deserialize_int(2).map(|v| cx.number(-(v as f64)).as_value(cx)),
            TypeTag::Int24Pos => self.deserialize_int(3).map(|v| cx.number(v as f64).as_value(cx)),
            TypeTag::Int24Neg => self.deserialize_int(3).map(|v| cx.number(-(v as f64)).as_value(cx)),
            TypeTag::Int32Pos => self.deserialize_int(4).map(|v| cx.number(v as f64).as_value(cx)),
            TypeTag::Int32Neg => self.deserialize_int(4).map(|v| cx.number(-(v as f64)).as_value(cx)),
            TypeTag::Int64Pos => self.deserialize_int(7).map(|v| cx.number(v as f64).as_value(cx)),
            TypeTag::Int64Neg => self.deserialize_int(7).map(|v| cx.number(-(v as f64)).as_value(cx)),

            TypeTag::Float => self.deserialize_f64().map(|v| cx.number(v).as_value(cx)),
            TypeTag::FloatStrPos => self.deserialize_f64_from_str().map(|v| cx.number(v).as_value(cx)),
            TypeTag::FloatStrNeg => self.deserialize_f64_from_str().map(|v| cx.number(-v).as_value(cx)),

            TypeTag::True => Ok(cx.boolean(true).as_value(cx)),
            TypeTag::False => Ok(cx.boolean(false).as_value(cx)),

            TypeTag::Str8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_string(len as usize, cx)
            }
            TypeTag::Str16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_string(len as usize, cx)
            }
            TypeTag::Str24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_string(len as usize, cx)
            }

            TypeTag::Map8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_map(len as usize, cx)
            }
            TypeTag::Map16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_map(len as usize, cx)
            }
            TypeTag::Map24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_map(len as usize, cx)
            }

            TypeTag::Array8 => {
                let len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                self.deserialize_array(len as usize, cx)
            }
            TypeTag::Array16 => {
                let len = self.deserialize_int(2)?;
                self.deserialize_array(len as usize, cx)
            }
            TypeTag::Array24 => {
                let len = self.deserialize_int(3)?;
                self.deserialize_array(len as usize, cx)
            }

            TypeTag::Mixed8 => {
                let array_len = self.reader.read_u8().ok_or("Unexpected EOF")?;
                let map_len = self.reader.read_u8().ok_or("Unexpected EOF")?;

                self.deserialize_mixed(array_len as usize, map_len as usize, cx)
            }
            TypeTag::Mixed16 => {
                let array_len = self.deserialize_int(2)?;
                let map_len = self.deserialize_int(2)?;

                self.deserialize_mixed(array_len as usize, map_len as usize, cx)
            }
            TypeTag::Mixed24 => {
                let array_len = self.deserialize_int(3)?;
                let map_len = self.deserialize_int(3)?;

                self.deserialize_mixed(array_len as usize, map_len as usize, cx)
            }

            TypeTag::StrRef8 => {
                let index = self.reader.read_u8().ok_or("Unexpected EOF")? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(cx.string(s.clone()).as_value(cx)),
                }
            }
            TypeTag::StrRef16 => {
                let index = self.deserialize_int(2)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(cx.string(s.clone()).as_value(cx)),
                }
            }
            TypeTag::StrRef24 => {
                let index = self.deserialize_int(3)? - 1;
                match self.string_refs.get(index as usize) {
                    None => Err("Invalid string reference"),
                    Some(s) => Ok(cx.string(s.clone()).as_value(cx)),
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

    fn deserialize_string<'c, C: 'c>(&mut self, len: usize, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        match self.reader.read_string(len) {
            None => Err("Unexpected EOF"),
            Some(s) => {
                let s = s.into_owned();
                if len > 2 {
                    self.string_refs.push(s.clone());
                }

                Ok(cx.string(s).as_value(cx))
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

    fn deserialize_map<'c, C: 'c>(&mut self, len: usize, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        let map = JsObject::new(cx);

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
                let (key, value) = (self.extract_value(cx)?, self.extract_value(cx)?);

                map.set(cx, key, value).unwrap();
            }
        }

        let map = map.as_value(cx);
        self.table_refs.push(map.clone());
        Ok(map)
    }

    fn deserialize_array<'c, C: 'c>(&mut self, len: usize, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        let v = cx.empty_array();

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

        for i in 0..len {
            check_recursion! {
                let index = cx.number(i as f64);
                let value = self.extract_value(cx)?;

                v.set(cx, index, value).unwrap();
            }
        }

        let v = v.as_value(cx);
        self.table_refs.push(v.clone());
        Ok(v)
    }

    fn deserialize_mixed<'c, C: 'c>(&mut self, array_len: usize, map_len: usize, cx: &'c mut C) -> Result<Handle<'v, JsValue>, &'static str>
    where
        C: Context<'v>,
    {
        let map = JsObject::new(cx);

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
                let index = cx.number(i as f64);
                let value = self.extract_value(cx)?;

                map.set(cx, index, value).unwrap();
            }
        }

        for _ in 0..map_len {
            check_recursion! {
                let (key, value) = (self.extract_value(cx)?, self.extract_value(cx)?);

                map.set(cx, key, value).unwrap();
            }
        }

        let map = map.as_value(cx);
        self.table_refs.push(map.clone());
        Ok(map)
    }
}
