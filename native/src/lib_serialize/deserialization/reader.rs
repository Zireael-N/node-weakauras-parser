use std::borrow::Cow;

pub(crate) struct SliceReader<'s> {
    buffer: &'s [u8],
    index: usize,
}

#[allow(dead_code)]
impl<'s> SliceReader<'s> {
    pub(crate) fn new(buffer: &'s [u8]) -> Self {
        Self { buffer, index: 0 }
    }

    #[inline]
    pub(crate) fn read_u8(&mut self) -> Option<u8> {
        if self.index < self.buffer.len() {
            let byte = self.buffer[self.index];
            self.index += 1;
            Some(byte)
        } else {
            None
        }
    }

    pub(crate) fn read_f64(&mut self) -> Option<f64> {
        if self.index + 7 < self.buffer.len() {
            let mut buf = [0; 8];
            buf.copy_from_slice(&self.buffer[self.index..self.index + 8]);
            self.index += 8;

            Some(f64::from_be_bytes(buf))
        } else {
            None
        }
    }

    pub(crate) fn read_int(&mut self, bytes: usize) -> Option<u64> {
        if (bytes > 0 && bytes <= 8) && (self.index + bytes - 1 < self.buffer.len()) {
            let mut buf = [0; 8];
            buf[8 - bytes..].copy_from_slice(&self.buffer[self.index..self.index + bytes]);
            self.index += bytes;

            Some(u64::from_be_bytes(buf))
        } else {
            None
        }
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> Option<&'s [u8]> {
        if self.index + len - 1 < self.buffer.len() {
            let bytes = &self.buffer[self.index..self.index + len];
            self.index += len;

            Some(bytes)
        } else {
            None
        }
    }

    pub(crate) fn read_string(&mut self, len: usize) -> Option<Cow<'s, str>> {
        if self.index + len - 1 < self.buffer.len() {
            let s = String::from_utf8_lossy(&self.buffer[self.index..self.index + len]);
            self.index += len;

            Some(s)
        } else {
            None
        }
    }
}
