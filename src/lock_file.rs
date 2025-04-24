use {
    anyhow::Result,
    std::{
        io,
        io::Write,
        path::{Path, PathBuf},
    },
    tempfile::{NamedTempFile, TempPath},
};

#[derive(Debug)]
pub struct LockFile {
    file: NamedTempFile,
}

impl LockFile {
    pub fn path(&self) -> &Path {
        self.file.path()
    }

    pub fn resource_path(&self) -> PathBuf {
        self.file.path().with_extension("")
    }

    pub fn acquire(path: impl AsRef<Path>) -> Result<Self> {
        Self::_acquire(path.as_ref())
    }

    fn _acquire(path: &Path) -> Result<Self> {
        Ok(Self {
            file: tempfile::Builder::new()
                .prefix(path.file_name().unwrap())
                .rand_bytes(0)
                .suffix(".lock")
                .tempfile_in(path.parent().unwrap())?,
        })
    }

    pub fn close(self) -> LockPath {
        LockPath {
            path: self.file.into_temp_path(),
        }
    }

    pub fn commit(self) -> Result<()> {
        self.close().commit()
    }
}

impl Write for LockFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[derive(Debug)]
pub struct LockPath {
    path: TempPath,
}

impl LockPath {
    pub fn resource_path(&self) -> PathBuf {
        self.path.with_extension("")
    }

    pub fn commit(self) -> Result<()> {
        let path = self.path.with_extension("");
        self.path.persist(path)?;
        Ok(())
    }
}
