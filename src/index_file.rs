use {
    crate::{hash, index::Index, lock_file::LockFile},
    anyhow::Result,
    memmap2::Mmap,
    std::{
        fs::File,
        io::BufWriter,
        ops::{Deref, DerefMut},
        path::PathBuf,
    },
};

#[derive(Debug)]
pub struct IndexFile {
    path: PathBuf,
    hash_kind: hash::Kind,
}

impl IndexFile {
    pub fn new(path: impl Into<PathBuf>, hash_kind: hash::Kind) -> Self {
        Self::_new(path.into(), hash_kind)
    }

    fn _new(path: PathBuf, hash_kind: hash::Kind) -> Self {
        Self { path, hash_kind }
    }

    pub fn read(&self) -> Result<Index> {
        let file = File::open(&self.path)?;
        let data = unsafe { Mmap::map(&file)? };
        Index::from_bytes(&data, self.hash_kind)
    }

    pub fn read_for_update(&self) -> Result<LockedIndex> {
        Ok(LockedIndex {
            lock_file: LockFile::acquire(&self.path)?,
            hash_kind: self.hash_kind,
            index: self.read()?,
        })
    }
}

#[derive(Debug)]
pub struct LockedIndex {
    lock_file: LockFile,
    hash_kind: hash::Kind,
    index: Index,
}

impl LockedIndex {
    pub fn commit(self) -> Result<()> {
        let mut writer = BufWriter::new(self.lock_file);
        self.index.write_to(&mut writer, self.hash_kind)?;
        writer.into_inner()?.commit()
    }
}

impl Deref for LockedIndex {
    type Target = Index;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl DerefMut for LockedIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.index
    }
}
