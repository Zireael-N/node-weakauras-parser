#[derive(Clone, Copy)]
pub(crate) struct Bitfield {
    data: u64,
    len: u8,
}

#[allow(dead_code)]
impl Bitfield {
    pub(crate) fn new() -> Self {
        Self { data: 0, len: 0 }
    }

    #[inline(always)]
    pub(crate) fn get_data(&self) -> u64 {
        self.data
    }

    #[inline(always)]
    pub(crate) fn get_len(&self) -> u8 {
        self.len
    }

    pub(crate) fn insert(&mut self, byte: u8) -> Result<(), ()> {
        if self.len < 64 - 8 {
            self.data += u64::from(byte) << self.len as u64;
            self.len += 8;
            Ok(())
        } else {
            Err(())
        }
    }

    pub(crate) fn fill_from_iterator<'a>(&mut self, iter: &mut impl Iterator<Item = &'a u8>) {
        while self.len < 64 - 8 {
            if let Some(&byte) = iter.next() {
                self.data += u64::from(byte) << self.len as u64;
                self.len += 8;
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    pub(crate) fn peek_byte(&self) -> u8 {
        self.data as u8
    }

    #[inline(always)]
    pub(crate) fn peek_bits(&self, bits: u8) -> u64 {
        self.data & ((1 << bits) - 1)
    }

    pub(crate) fn discard_bits(&mut self, bits: u8) {
        self.data >>= bits as u64;
        self.len = self.len.saturating_sub(bits);
    }

    pub(crate) fn extract_bits(&mut self, bits: u8) -> u64 {
        let result = self.data & ((1 << bits as u64) - 1);
        self.data >>= bits as u64;
        self.len = self.len.saturating_sub(bits);
        result
    }

    pub(crate) fn extract_byte(&mut self) -> Result<u8, ()> {
        if self.len >= 8 {
            let result = self.data as u8;
            self.data >>= 8;
            self.len -= 8;
            Ok(result)
        } else {
            Err(())
        }
    }

    pub(crate) fn insert_and_extract_byte(&mut self, byte: u8) -> u8 {
        if self.len <= 64 - 8 {
            self.data += u64::from(byte) << self.len as u64;
            let result = self.data as u8;
            self.data >>= 8;
            result
        } else {
            let result = self.data as u8;
            self.data >>= 8;
            self.data += u64::from(byte) << (self.len - 8) as u64;
            result
        }
    }
}
