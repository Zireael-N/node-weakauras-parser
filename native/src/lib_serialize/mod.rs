pub(crate) mod deserialization;
pub(crate) mod type_tag;

pub(crate) use deserialization::Deserializer;
pub(crate) use type_tag::{EmbeddedTypeTag, TypeTag};

#[allow(dead_code)]
pub(crate) const MAJOR: &str = "LibSerialize";
#[allow(dead_code)]
pub(crate) const MINOR: u8 = 1;
