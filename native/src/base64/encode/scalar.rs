use crate::base64::byte_map::ENCODE_LUT;

#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
pub(crate) unsafe fn encode(data: &[u8], buf: &mut String) {
    let mut chunks = data.chunks_exact(3);

    let mut len = buf.len();
    let mut ptr = buf[len..].as_mut_ptr();
    for chunk in chunks.by_ref() {
        len += 4;

        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        ptr.write(ENCODE_LUT[b0]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[(b0 >> 6) | (b1 << 2)]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[(b1 >> 4) | (b2 << 4)]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[b2 >> 2]);
        ptr = ptr.add(1);
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        2 => {
            len += 3;
            let b0 = remainder[0];
            let b1 = remainder[1];

            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b0 >> 6) | (b1 << 2)]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[b1 >> 4]);
        }
        1 => {
            len += 2;
            let b0 = remainder[0];

            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[b0 >> 6]);
        }
        _ => (),
    }

    buf.as_mut_vec().set_len(len);
}
