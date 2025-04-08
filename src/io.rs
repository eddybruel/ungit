use {
    crate::{hash, hash::Hash},
    std::{io, io::Write},
};

pub trait WriteExt: Write {
    fn write_u16(&mut self, value: u16) -> io::Result<()>;

    fn write_u32(&mut self, value: u32) -> io::Result<()>;
}

impl<W> WriteExt for W
where
    W: Write,
{
    fn write_u16(&mut self, value: u16) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }
}

#[derive(Debug)]
pub struct CountWriter<W> {
    inner: W,
    count: usize,
}

impl<W> CountWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, count: 0 }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl<W> Write for CountWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let num_written = self.inner.write(buf)?;
        self.count += num_written;
        Ok(num_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug)]
pub struct HashWriter<W> {
    inner: W,
    hash: hash::Builder,
}

impl<W> HashWriter<W> {
    pub fn new(inner: W, hash_kind: hash::Kind) -> Self {
        Self {
            inner,
            hash: hash::Builder::new(hash_kind),
        }
    }

    pub fn finish(self) -> (W, Hash) {
        (self.inner, self.hash.finish())
    }
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.hash.write(&buf[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
