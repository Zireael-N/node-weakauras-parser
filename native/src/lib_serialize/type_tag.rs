#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) enum EmbeddedTypeTag {
    Str = 0,
    Map = 1,
    Array = 2,
    Mixed = 3,
}

impl EmbeddedTypeTag {
    pub(crate) fn from_u8(v: u8) -> Option<Self> {
        if v <= 3 {
            // SAFETY: we've checked that `v` can be represented as `EmbeddedTypeTag`
            Some(unsafe { std::mem::transmute::<u8, EmbeddedTypeTag>(v) })
        } else {
            None
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        // SAFETY: safe due to #[repr(u8)]
        unsafe { std::mem::transmute(self) }
    }
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) enum TypeTag {
    Null = 0,

    Int16Pos = 1,
    Int16Neg = 2,
    Int24Pos = 3,
    Int24Neg = 4,
    Int32Pos = 5,
    Int32Neg = 6,
    Int64Pos = 7,
    Int64Neg = 8,

    Float = 9,
    FloatStrPos = 10,
    FloatStrNeg = 11,

    True = 12,
    False = 13,

    Str8 = 14,
    Str16 = 15,
    Str24 = 16,

    Map8 = 17,
    Map16 = 18,
    Map24 = 19,

    Array8 = 20,
    Array16 = 21,
    Array24 = 22,

    Mixed8 = 23,
    Mixed16 = 24,
    Mixed24 = 25,

    StrRef8 = 26,
    StrRef16 = 27,
    StrRef24 = 28,

    MapRef8 = 29,
    MapRef16 = 30,
    MapRef24 = 31,
}

impl TypeTag {
    pub(crate) fn from_u8(v: u8) -> Option<Self> {
        if v <= 31 {
            // SAFETY: we've checked that `v` can be represented as `TypeTag`
            Some(unsafe { std::mem::transmute::<u8, TypeTag>(v) })
        } else {
            None
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        // SAFETY: safe due to #[repr(u8)]
        unsafe { std::mem::transmute(self) }
    }
}
