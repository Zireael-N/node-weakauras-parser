mod scalar;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
mod sse;

#[inline(always)]
fn calculate_capacity(s: &str) -> Result<usize, &'static str> {
    // Equivalent to s.len() * 3 / 4 but does not overflow
    let len = s.len();

    let leftover = len % 4;
    if leftover == 1 {
        return Err("Invalid base64 length");
    }
    let mut result = len / 4 * 3;

    if leftover > 0 {
        result += leftover - 1;
    }

    Ok(result)
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let mut buffer = Vec::with_capacity(calculate_capacity(s)?);
    unsafe {
        sse::decode(s.as_bytes(), &mut buffer)?;
    }
    Ok(buffer)
}

#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(target_feature = "ssse3")
))]
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let mut buffer = Vec::with_capacity(calculate_capacity(s)?);
    unsafe {
        scalar::decode(s.as_bytes(), &mut buffer)?;
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    ))]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = (b'0'..=b'9')
            .chain(b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'('..=b')')
            .cycle()
            .take(1024 * 1024 + 3)
            .collect();

        let cap = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(cap);
        let mut buf2 = Vec::with_capacity(cap);

        unsafe {
            scalar::decode(&data, &mut buf1).unwrap();
            sse::decode(&data, &mut buf2).unwrap();
        }

        assert_eq!(buf1, buf2);
    }
}
