use {
    anyhow::{Result, anyhow},
    bstr::ByteSlice,
    std::ops::{Deref, DerefMut},
};

#[derive(Clone, Debug)]
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining_len(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub fn remaining_bytes(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }

    pub fn skip_bytes(&mut self, count: usize) -> bool {
        if self.remaining_len() < count {
            return false;
        }
        self.pos += count;
        true
    }

    pub fn skip_bytes_if_matches(&mut self, bytes: &[u8]) -> bool {
        if !self.remaining_bytes().starts_with(bytes) {
            return false;
        }
        self.pos += bytes.len();
        true
    }

    pub fn skip_bytes_until_end(&mut self) -> usize {
        let count = self.remaining_len();
        self.pos += count;
        count
    }

    pub fn skip_bytes_until_found(&mut self, byte: u8) -> Option<usize> {
        let Some(count) = self.remaining_bytes().find_byte(byte) else {
            return None;
        };
        self.pos += count;
        Some(count)
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        let mut reader = self.track();
        if !reader.skip_bytes(count) {
            return Err(anyhow!("unexpected end of file"));
        }
        Ok(reader.bytes())
    }

    pub fn read_bytes_until_end(&mut self) -> &'a [u8] {
        let mut reader = self.track();
        reader.skip_bytes_until_end();
        reader.bytes()
    }

    pub fn read_bytes_until_found(&mut self, byte: u8) -> Result<&'a [u8]> {
        let mut reader = self.track();
        if !reader.skip_bytes_until_found(byte).is_none() {
            return Err(anyhow!("unexpected end of file"));
        };
        Ok(reader.bytes())
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    pub fn track(&mut self) -> Track<'_, 'a> {
        let start = self.pos;
        Track {
            reader: self,
            start,
        }
    }
}

#[derive(Debug)]
pub struct Track<'a, 'b> {
    reader: &'a mut Reader<'b>,
    start: usize,
}

impl<'a> Track<'_, 'a> {
    pub fn bytes(&self) -> &'a [u8] {
        &self.reader.bytes[self.start..self.reader.position()]
    }
}

impl<'a> Deref for Track<'_, 'a> {
    type Target = Reader<'a>;

    fn deref(&self) -> &Self::Target {
        self.reader
    }
}

impl<'a> DerefMut for Track<'_, 'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reader
    }
}
