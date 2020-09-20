// Based on SliceRead from serde_json
// https://github.com/serde-rs/json
// Licensed under either of Apache License, Version 2.0
// or MIT license at your option (see LICENSES/serde_json).

pub(crate) struct StrReader<'s> {
    buffer: &'s [u8],
    index: usize,
    scratch: Vec<u8>,
}

#[allow(dead_code)]
impl<'s> StrReader<'s> {
    pub(crate) fn new(buffer: &'s str) -> Self {
        Self {
            buffer: buffer.as_bytes(),
            index: 0,
            scratch: Vec::new(),
        }
    }

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.index < self.buffer.len() {
            let c = self.buffer[self.index];
            self.index += 1;
            Some(c)
        } else {
            None
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        if self.index < self.buffer.len() {
            Some(self.buffer[self.index])
        } else {
            None
        }
    }

    #[inline]
    fn discard(&mut self) {
        self.index += 1;
    }

    #[inline]
    pub(crate) fn position(&self) -> usize {
        self.index
    }

    pub(crate) fn read_identifier(&mut self) -> Result<&str, &'static str> {
        if self.index + 1 < self.buffer.len() {
            // Matching against the second byte to ensure
            // we don't break up a multibyte character.
            match (self.buffer[self.index], self.buffer[self.index + 1]) {
                (b'^', 0x00..=0x79) => {
                    let result = unsafe { std::str::from_utf8_unchecked(&self.buffer[self.index..self.index + 2]) };
                    self.index += 2;
                    Ok(result)
                }
                _ => Err("Not an identifier"),
            }
        } else {
            Err("Unexpected EOF")
        }
    }

    pub(crate) fn peek_identifier(&self) -> Result<&str, &'static str> {
        if self.index + 1 < self.buffer.len() {
            // Matching against the second byte to ensure
            // we don't break up a multibyte character.
            match (self.buffer[self.index], self.buffer[self.index + 1]) {
                (b'^', 0x00..=0x79) => {
                    Ok(unsafe { std::str::from_utf8_unchecked(&self.buffer[self.index..self.index + 2]) })
                }
                _ => Err("Not an identifier"),
            }
        } else {
            Err("Unexpected EOF")
        }
    }

    pub(crate) fn read_until_next(&mut self) -> Result<&str, &'static str> {
        let start = self.index;

        loop {
            match self.peek() {
                None => return Err("Unexpected EOF"),
                Some(b'^') => {
                    // SAFETY: As long as `start` does not point at the middle
                    // of a multibyte character, this should be safe.
                    // Public API does not allow the reader to end up in such a state.
                    return Ok(unsafe { std::str::from_utf8_unchecked(&self.buffer[start..self.index]) });
                }
                _ => self.discard(),
            }
        }
    }

    pub(crate) fn parse_str(&mut self) -> Result<&str, &'static str> {
        self.scratch.clear();

        let mut copy_from = self.index;

        loop {
            match self.peek() {
                None => return Err("Unexpected EOF"),
                Some(b'^') => {
                    if self.scratch.is_empty() {
                        // SAFETY: As long as `copy_from` does not point at the middle
                        // of a multibyte character, this should be safe.
                        // Public API does not allow the reader to end up in such a state.
                        return Ok(unsafe { std::str::from_utf8_unchecked(&self.buffer[copy_from..self.index]) });
                    } else {
                        // SAFETY: None of the replaced bytes and their replacements
                        // has the most significant bit set to 1.
                        self.scratch.extend_from_slice(&self.buffer[copy_from..self.index]);
                        return Ok(unsafe { std::str::from_utf8_unchecked(&self.scratch) });
                    }
                }
                Some(b'~') => {
                    self.scratch.extend_from_slice(&self.buffer[copy_from..self.index]);

                    self.discard();

                    let replacement = match self.peek() {
                        Some(v @ 0x40..=0x5D) | Some(v @ 0x5F..=0x60) => v - 64,
                        Some(0x7A) => 0x1E,
                        Some(0x7B) => 0x7F,
                        Some(0x7C) => 0x7E,
                        Some(0x7D) => 0x5E,
                        _ => return Err("Invalid escape character"),
                        // _ => continue,
                    };

                    self.discard();
                    self.scratch.push(replacement);

                    copy_from = self.index;
                }
                _ => self.discard(),
            }
        }
    }
}
