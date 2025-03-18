use {
    anyhow::{Result, anyhow},
    bstr::ByteSlice,
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

    pub fn remaining_bytes(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }

    pub fn read(&mut self, count: usize) -> Result<&'a [u8]> {
        if count > self.remaining_bytes().len() {
            return Err(anyhow!("unexpected end of file"));
        }
        let bytes = &self.remaining_bytes()[..count];
        self.pos += count;
        Ok(bytes)
    }

    pub fn read_until(&mut self, byte: u8) -> Result<&'a [u8]> {
        let Some(count) = self.remaining_bytes().find_byte(byte) else {
            return Err(anyhow!("unexpected end of file"));
        };
        self.read(count)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.read(2)?.try_into().unwrap()))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }
}
