mod scalar;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
mod sse;

const OVERFLOW_ERROR: &str = "Cannot calculate capacity without overflowing";

#[inline(always)]
fn calculate_capacity(data: &[u8]) -> Option<usize> {
    // Equivalent to (s.len() * 4 + 2) / 3 but avoids an early overflow
    let len = data.len();
    let leftover = len % 3;

    (len / 3).checked_mul(4).and_then(|len| {
        if leftover > 0 {
            len.checked_add(leftover + 1)
        } else {
            Some(len)
        }
    })
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
unsafe fn encode(data: &[u8], buf: &mut String) {
    unsafe {
        sse::encode(data, buf);
    }
}

#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(target_feature = "ssse3")
))]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
unsafe fn encode(data: &[u8], buf: &mut String) {
    unsafe {
        scalar::encode(data, buf);
    }
}

/// Same as encode_raw() but prepends the output with the provided prefix
/// to reduce allocations.
pub(crate) fn encode_with_prefix(data: &[u8], prefix: &str) -> Result<String, &'static str> {
    let mut result = String::with_capacity(
        calculate_capacity(data)
            .and_then(|len| len.checked_add(prefix.len()))
            .ok_or(OVERFLOW_ERROR)?,
    );
    result.push_str(prefix);

    unsafe {
        encode(data, &mut result);
    }
    Ok(result)
}

#[allow(dead_code)]
pub(crate) fn encode_raw(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data).ok_or(OVERFLOW_ERROR)?);

    unsafe {
        encode(data, &mut result);
    }
    Ok(result)
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
