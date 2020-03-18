mod scalar;
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
mod sse;

#[inline(always)]
fn calculate_capacity(data: &[u8]) -> Result<usize, &'static str> {
    // Equivalent to (s.len() * 4 + 2) / 3 but avoids an early overflow
    let len = data.len();
    let leftover = len % 3;

    (len / 3)
        .checked_mul(4)
        .and_then(|len| {
            if leftover > 0 {
                len.checked_add(leftover + 1)
            } else {
                Some(len)
            }
        })
        .ok_or("cannot calculate capacity without overflowing")
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
unsafe fn encode(data: &[u8], buf: &mut String) {
    sse::encode(data, buf);
}

#[cfg(any(not(any(target_arch = "x86", target_arch = "x86_64")), not(target_feature = "ssse3")))]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
unsafe fn encode(data: &[u8], buf: &mut String) {
    scalar::encode(data, buf);
}

/// Same as encode_raw() but prepends the output with "!"
/// to reduce allocations.
pub(crate) fn encode_weakaura(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data)? + 1);
    result.push_str("!");

    unsafe {
        encode(data, &mut result);
    }
    Ok(result)
}

#[allow(dead_code)]
pub(crate) fn encode_raw(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data)?);

    unsafe {
        encode(data, &mut result);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "ssse3"))]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let cap = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(cap);
        let mut buf2 = String::with_capacity(cap);

        unsafe {
            scalar::encode(&data, &mut buf1);
            sse::encode(&data, &mut buf2);
        }

        assert_eq!(buf1, buf2);
    }
}
