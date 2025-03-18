use {
    anyhow::Result,
    std::{io, io::Write, path::Path},
    tempfile::NamedTempFile,
};

#[derive(Debug)]
pub struct Lockfile {
    tempfile: NamedTempFile,
}

impl Lockfile {
    pub fn acquire(path: impl AsRef<Path>) -> Result<Self> {
        Self::_acquire(path.as_ref())
    }

    fn _acquire(path: &Path) -> Result<Self> {
        Ok(Self {
            tempfile: tempfile::Builder::new()
                .prefix(path.file_name().unwrap())
                .rand_bytes(0)
                .suffix(".lock")
                .tempfile_in(path.parent().unwrap())?,
        })
    }

    pub fn commit(self) -> Result<()> {
        let path = self.tempfile.path().with_extension("");
        self.tempfile.persist(path)?;
        Ok(())
    }
}

impl Write for Lockfile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tempfile.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tempfile.flush()
    }
}
