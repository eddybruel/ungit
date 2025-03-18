use {
    anyhow::Result,
    crate::{object, oid, oid::Oid, io::HashWriter},
    flate2::{Compression, write::ZlibEncoder},
    std::{io, io::Write, path::PathBuf},
    tempfile::NamedTempFile,
};

#[derive(Debug)]
pub struct Odb {
    path: PathBuf,
    oid_kind: oid::Kind,
}

impl Odb {
    pub fn new(path: impl Into<PathBuf>, oid_kind: oid::Kind) -> Self {
        Self::_new(path.into(), oid_kind)
    }

    fn _new(path: PathBuf, oid_kind: oid::Kind) -> Self {
        Self { path, oid_kind }
    }

    pub fn store(&self, kind: object::Kind, content: &[u8]) -> Result<Oid> {
        let mut writer = self.store_streaming(kind, content.len())?;
        writer.write_all(content)?;
        writer.commit()
    }

    pub fn store_streaming(&self, kind: object::Kind, len: usize) -> Result<Writer> {
        let tempfile = NamedTempFile::new_in(&self.path)?;
        let encoder = ZlibEncoder::new(tempfile, Compression::default());
        let inner = HashWriter::new(encoder, self.oid_kind);
        let mut writer = Writer { inner };
        write!(writer, "{} {}\0", kind, len)?;
        Ok(writer)
    }
}

#[derive(Debug)]
pub struct Writer {
    inner: HashWriter<ZlibEncoder<NamedTempFile>>,
}

impl Writer {
    pub fn commit(self) -> Result<Oid> {
        let (encoder, oid) = self.inner.finish();
        let tempfile = encoder.finish()?;
        let mut path = tempfile.path().parent().unwrap().to_path_buf();
        let oid_string = oid.to_string();
        path.push(&oid_string[..2]);
        path.push(&oid_string[2..]);
        tempfile.persist(&path)?;
        Ok(oid)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
