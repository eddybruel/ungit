use {
    crate::{
        hash,
        lock_file::LockFile,
        ref_::Ref,
    },
    anyhow::Result,
    bstr::{BStr, BString},
    std::{collections::HashMap, ffi::OsStr, fs, os::unix::ffi::OsStrExt, path::PathBuf},
};

#[derive(Debug)]
pub struct RefStore {
    path: PathBuf,
    hash_kind: hash::Kind,
}

impl RefStore {
    pub fn new(path: impl Into<PathBuf>, hash_kind: hash::Kind) -> Self {
        Self::_new(path.into(), hash_kind)
    }

    fn _new(path: PathBuf, hash_kind: hash::Kind) -> Self {
        Self {
            path,
            hash_kind,
        }
    }

    pub fn create_transaction(&self) -> Transaction {
        Transaction {
            path: self.path.clone(),
            hash_kind: self.hash_kind,
            ops: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    path: PathBuf,
    hash_kind: hash::Kind,
    ops: Vec<Op>,
}

impl Transaction {
    pub fn create(&mut self, name: BString, new: Ref) {
        self.ops.push(Op::Create { name, new });
    }

    pub fn update(&mut self, name: BString, old: Ref, new: Ref) {
        self.ops.push(Op::Update { name, old, new });
    }

    pub fn delete(&mut self, name: BString, old: Ref) {
        self.ops.push(Op::Delete { name, old });
    }

    pub fn commit(self) -> Result<()> {
        let mut refs = HashMap::new();
        for op in &self.ops {
            let name = op.name();
            let path = self.path.join(OsStr::from_bytes(name));
            let mut lock_file = LockFile::acquire(&path)?;
            if let Some(old) = op.old() {
                let data = fs::read(path)?;
                let ref_ = Ref::from_bytes(&data, self.hash_kind)?;
                if &ref_ != old {
                    return Err(anyhow::anyhow!("reference {} has been modified", name));
                }
            } else if path.exists() {
                return Err(anyhow::anyhow!("reference {} already exists", name));
            }
            if let Some(new) = op.new() {
                new.write_to(&mut lock_file)?;
            };
            refs.insert(name, lock_file.close());
        }
        for op in &self.ops {
            let lock_file = refs.remove(op.name()).unwrap();
            if op.new().is_some() {
                lock_file.commit()?;
            } else {
                fs::remove_file(lock_file.resource_path())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum Op {
    Create { name: BString, new: Ref },
    Update { name: BString, old: Ref, new: Ref },
    Delete { name: BString, old: Ref },
}

impl Op {
    fn name(&self) -> &BStr {
        match self {
            Op::Create { name, .. } | Op::Update { name, .. } | Op::Delete { name, .. } => {
                name.as_ref()
            }
        }
    }

    fn old(&self) -> Option<&Ref> {
        match self {
            Op::Update { old, .. } | Op::Delete { old, .. } => Some(old),
            _ => None,
        }
    }

    fn new(&self) -> Option<&Ref> {
        match self {
            Op::Create { new, .. } | Op::Update { new, .. } => Some(new),
            _ => None,
        }
    }
}
