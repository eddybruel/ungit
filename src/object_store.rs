use {
    crate::{hash, hash::Hash, io::HashWriter, object, object::Object},
    anyhow::Result,
    flate2::{Compression, write::ZlibEncoder},
    std::{fs, io, io::Write, path::PathBuf},
    tempfile::NamedTempFile,
};

#[derive(Debug)]
pub struct ObjectStore {
    path: PathBuf,
    hash_kind: hash::Kind,
}

impl ObjectStore {
    pub fn new(path: impl Into<PathBuf>, hash_kind: hash::Kind) -> Self {
        Self::_new(path.into(), hash_kind)
    }

    fn _new(path: PathBuf, hash_kind: hash::Kind) -> Self {
        Self { path, hash_kind }
    }

    pub fn write(&self, object: Object<'_>) -> Result<Hash> {
        let mut writer = self.create_writer(object.header())?;
        writer.write_all(&object.content)?;
        writer.finish()
    }

    pub fn create_writer(&self, header: object::Header) -> Result<Writer> {
        let mut writer = Writer {
            inner: HashWriter::new(
                ZlibEncoder::new(NamedTempFile::new_in(&self.path)?, Compression::default()),
                self.hash_kind,
            ),
        };
        write!(writer, "{}", header)?;
        Ok(writer)
    }
}

#[derive(Debug)]
pub struct Writer {
    inner: HashWriter<ZlibEncoder<NamedTempFile>>,
}

impl Writer {
    pub fn finish(self) -> Result<Hash> {
        let (encoder, id) = self.inner.finish();
        let temp_file = encoder.finish()?;
        let mut path = temp_file.path().parent().unwrap().to_path_buf();
        let mut id_hex_bytes = [0; hash::Kind::LONGEST.hex_len()];
        let id_hex_str = id.to_hex_str(&mut id_hex_bytes);
        path.push(&id_hex_str[..2]);
        path.push(&id_hex_str[2..]);
        if let Err(error) = temp_file.persist(&path) {
            fs::create_dir(path.parent().unwrap())?;
            error.file.persist(&path)?;
        }
        Ok(id)
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
