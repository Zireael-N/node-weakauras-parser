mod byte_map;
mod decode;
mod encode;
pub(crate) use decode::decode;
#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use encode::{encode_raw, encode_with_prefix};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_is_inverse_of_encode() {
        let data: Vec<u8> = (0..=255).cycle().take(1000).collect();

        for chunk in data.chunks_exact(100) {
            let encoded = encode_raw(chunk).unwrap();

            assert_eq!(decode(&encoded).unwrap(), chunk);
        }
    }
}
