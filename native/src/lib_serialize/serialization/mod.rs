use super::{EmbeddedTypeTag, TypeTag, MINOR};
use indexmap::IndexMap;
use serde_json::{map::Map, Value};

const TYPE_TAG_SHIFT: u8 = 3;
const EMBEDDED_TYPE_TAG_SHIFT: u8 = 2;
const EMBEDDED_LEN_SHIFT: u8 = 4;

fn required_bytes(v: u64) -> u8 {
    // `match` does not support non-inclusive ranges as of rustc 1.46
    if v < 256 {
        1
    } else if v < 65_536 {
        2
    } else if v < 16_777_216 {
        3
    } else if v < 4_294_967_296 {
        4
    } else {
        7
    }
}

pub struct Serializer {
    remaining_depth: usize,
    result: Vec<u8>,

    string_refs: IndexMap<String, usize>,
}

impl Serializer {
    pub fn serialize(mut value: Value, approximate_len: Option<usize>) -> Result<Vec<u8>, &'static str> {
        let mut serializer = Self {
            remaining_depth: 128,
            result: Vec::with_capacity(approximate_len.unwrap_or(1024)),

            string_refs: IndexMap::new(),
        };

        serializer.result.push(MINOR);
        serializer.serialize_helper(&mut value)?;

        Ok(serializer.result)
    }

    fn serialize_helper(&mut self, value: &mut Value) -> Result<(), &'static str> {
        match *value {
            Value::Null => self.serialize_null(),
            Value::Bool(b) => {
                if b {
                    self.result.push(TypeTag::True.to_u8() << TYPE_TAG_SHIFT);
                } else {
                    self.result.push(TypeTag::False.to_u8() << TYPE_TAG_SHIFT);
                }
            }
            Value::String(ref s) => self.serialize_string(s)?,
            Value::Number(ref n) => match n.as_f64() {
                Some(n) => self.serialize_number(n),
                None => self.serialize_null(),
            },
            Value::Array(ref mut v) => self.serialize_slice(v)?,
            Value::Object(ref mut m) => self.serialize_table(m)?,
        }

        Ok(())
    }

    #[inline(always)]
    fn serialize_null(&mut self) {
        self.result.push(TypeTag::Null.to_u8() << TYPE_TAG_SHIFT);
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::manual_range_contains))]
    fn serialize_number(&mut self, value: f64) {
        const MAX_7_BIT: i64 = 72_057_594_037_927_936 - 1; // 2^56 - 1, `i64::pow` is not a `const fn` as of rustc 1.46
        const MAX_7_BIT_FLOAT: f64 = MAX_7_BIT as f64;

        if value.fract() != 0.0 || (value < -MAX_7_BIT_FLOAT || value > MAX_7_BIT_FLOAT) {
            self.result.push(TypeTag::Float.to_u8() << TYPE_TAG_SHIFT);
            self.result.extend_from_slice(&value.to_be_bytes());
        } else {
            // SAFETY:
            // 1) for infinity and NaNs, `f64::fract()` returns `f64::NAN`;
            // 2) `value` does not have a fractional part;
            // 3) `value` is within i64::MIN..=i64::MAX range.
            let value = unsafe { value.to_int_unchecked::<i64>() };

            if value > -4096 && value < 4096 {
                if value > 0 && value < 128 {
                    self.result.push(((value as u8) << 1) | 1);
                } else {
                    let (value, neg_bit) = if value < 0 {
                        (-value, 1 << TYPE_TAG_SHIFT)
                    } else {
                        (value, 0)
                    };

                    let value = (value << 4) | neg_bit | 4;
                    self.result.push(value as u8);
                    self.result.push((value >> 8) as u8);
                }
            } else {
                let (value, neg_bit) = if value < 0 {
                    ((-value) as u64, 1)
                } else {
                    (value as u64, 0)
                };

                match required_bytes(value) {
                    2 => {
                        self.result
                            .push((TypeTag::Int16Pos.to_u8() + neg_bit) << TYPE_TAG_SHIFT);
                        self.serialize_int(value, 2);
                    }
                    3 => {
                        self.result
                            .push((TypeTag::Int24Pos.to_u8() + neg_bit) << TYPE_TAG_SHIFT);
                        self.serialize_int(value, 3);
                    }
                    4 => {
                        self.result
                            .push((TypeTag::Int32Pos.to_u8() + neg_bit) << TYPE_TAG_SHIFT);
                        self.serialize_int(value, 4);
                    }
                    _ => {
                        self.result
                            .push((TypeTag::Int64Pos.to_u8() + neg_bit) << TYPE_TAG_SHIFT);
                        self.serialize_int(value, 7);
                    }
                }
            }
        }
    }

    fn serialize_int(&mut self, value: u64, len: usize) {
        let bytes = value.to_be_bytes();
        self.result.extend_from_slice(&bytes[bytes.len() - len..]);
    }

    fn serialize_string(&mut self, value: &str) -> Result<(), &'static str> {
        match self.string_refs.get(value) {
            Some(index) => {
                let index = *index as u64;

                match required_bytes(index) {
                    1 => {
                        self.result.push(TypeTag::StrRef8.to_u8() << TYPE_TAG_SHIFT);
                        self.serialize_int(index, 1);
                    }
                    2 => {
                        self.result.push(TypeTag::StrRef16.to_u8() << TYPE_TAG_SHIFT);
                        self.serialize_int(index, 2);
                    }
                    3 => {
                        self.result.push(TypeTag::StrRef24.to_u8() << TYPE_TAG_SHIFT);
                        self.serialize_int(index, 3);
                    }
                    _ => return Err("Serialization error: more than 2^24 different strings"),
                }
            }
            None => {
                let len = value.len();

                if len < 16 {
                    self.result.push(
                        (EmbeddedTypeTag::Str.to_u8() << EMBEDDED_TYPE_TAG_SHIFT)
                            | ((len as u8) << EMBEDDED_LEN_SHIFT)
                            | 2,
                    );
                } else {
                    let len = value.len() as u64;

                    match required_bytes(len) {
                        1 => {
                            self.result.push(TypeTag::Str8.to_u8() << TYPE_TAG_SHIFT);
                            self.serialize_int(len, 1);
                        }
                        2 => {
                            self.result.push(TypeTag::Str16.to_u8() << TYPE_TAG_SHIFT);
                            self.serialize_int(len, 2);
                        }
                        3 => {
                            self.result.push(TypeTag::Str24.to_u8() << TYPE_TAG_SHIFT);
                            self.serialize_int(len, 3);
                        }
                        _ => return Err("Serialization error: string is too large"),
                    }
                }

                if len > 2 {
                    self.string_refs.insert(value.into(), self.string_refs.len() + 1);
                }

                self.result.extend_from_slice(&value.as_bytes());
            }
        }

        Ok(())
    }

    fn serialize_table(&mut self, map: &mut Map<String, Value>) -> Result<(), &'static str> {
        let mut array = Vec::new();
        let mut index = 1;

        while let Some(el) = map.remove(&index.to_string()) {
            array.push(el);
            index += 1;
        }

        if map.is_empty() {
            self.serialize_slice(&mut array)
        } else if array.is_empty() {
            self.serialize_map(map)
        } else {
            self.serialize_mixed(&mut array, map)
        }
    }

    fn serialize_slice(&mut self, slice: &mut [Value]) -> Result<(), &'static str> {
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

        let len = slice.len();
        if len < 16 {
            self.result.push(
                (EmbeddedTypeTag::Array.to_u8() << EMBEDDED_TYPE_TAG_SHIFT) | ((len as u8) << EMBEDDED_LEN_SHIFT) | 2,
            );
        } else {
            let len = len as u64;
            match required_bytes(len) {
                1 => {
                    self.result.push(TypeTag::Array8.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 1);
                }
                2 => {
                    self.result.push(TypeTag::Array16.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 2);
                }
                3 => {
                    self.result.push(TypeTag::Array24.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 3);
                }
                _ => return Err("Serialization error: array is too large"),
            }
        }

        for el in slice {
            check_recursion! {
                self.serialize_helper(el)?;
            }
        }

        Ok(())
    }

    fn serialize_map(&mut self, map: &mut Map<String, Value>) -> Result<(), &'static str> {
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

        let len = map.len();

        if len < 16 {
            self.result.push(
                (EmbeddedTypeTag::Map.to_u8() << EMBEDDED_TYPE_TAG_SHIFT) | ((len as u8) << EMBEDDED_LEN_SHIFT) | 2,
            );
        } else {
            let len = len as u64;
            match required_bytes(len) {
                1 => {
                    self.result.push(TypeTag::Map8.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 1);
                }
                2 => {
                    self.result.push(TypeTag::Map16.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 2);
                }
                3 => {
                    self.result.push(TypeTag::Map24.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(len, 3);
                }
                _ => return Err("Serialization error: map is too large"),
            }
        }

        for (key, value) in map {
            check_recursion! {
                self.serialize_string(&key)?;
                self.serialize_helper(value)?;
            }
        }

        Ok(())
    }

    fn serialize_mixed(&mut self, slice: &mut [Value], map: &mut Map<String, Value>) -> Result<(), &'static str> {
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

        let map_len = map.len();
        let slice_len = slice.len();

        if map_len < 5 && slice_len < 5 {
            let packed_len = ((map_len as u8 - 1) << 2) | (slice_len as u8 - 1);
            self.result.push(
                (EmbeddedTypeTag::Mixed.to_u8() << EMBEDDED_TYPE_TAG_SHIFT) | (packed_len << EMBEDDED_LEN_SHIFT) | 2,
            );
        } else {
            let (map_len, slice_len) = (map_len as u64, slice_len as u64);

            match required_bytes(std::cmp::max(map_len, slice_len)) {
                1 => {
                    self.result.push(TypeTag::Mixed8.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(slice_len, 1);
                    self.serialize_int(map_len, 1);
                }
                2 => {
                    self.result.push(TypeTag::Mixed16.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(slice_len, 2);
                    self.serialize_int(map_len, 2);
                }
                3 => {
                    self.result.push(TypeTag::Mixed24.to_u8() << TYPE_TAG_SHIFT);
                    self.serialize_int(slice_len, 3);
                    self.serialize_int(map_len, 3);
                }
                _ => return Err("Serialization error: map is too large"),
            }
        }

        for el in slice {
            check_recursion! {
                self.serialize_helper(el)?;
            }
        }

        for (key, value) in map {
            check_recursion! {
                self.serialize_string(&key)?;
                self.serialize_helper(value)?;
            }
        }

        Ok(())
    }
}
