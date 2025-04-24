use {
    crate::{hash, hash::Hash, ref_::Ref},
    anyhow::{Result, anyhow},
    bstr::BString,
    std::{
        collections::{HashMap, hash_map},
        path::PathBuf,
        sync::Arc,
    },
};

#[derive(Debug)]
pub struct RefStore {
    inner: Arc<RefStoreInner>,
}

impl RefStore {
    pub fn new(path: impl Into<PathBuf>, hash_kind: hash::Kind) -> Self {
        Self::_new(path.into(), hash_kind)
    }

    fn _new(path: PathBuf, hash_kind: hash::Kind) -> Self {
        Self {
            inner: Arc::new(RefStoreInner { path, hash_kind }),
        }
    }

    pub fn start_transaction(&self) -> Transaction {
        Transaction {
            ref_store_inner: self.inner.clone(),
            ops: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct RefStoreInner {
    path: PathBuf,
    hash_kind: hash::Kind,
}

impl RefStoreInner {
    fn hash_kind(&self) -> hash::Kind {
        self.hash_kind
    }
}

#[derive(Debug)]
pub struct Transaction {
    ref_store_inner: Arc<RefStoreInner>,
    ops: HashMap<BString, Op>,
}

impl Transaction {
    pub fn update(&mut self, name: BString, old: Option<Ref>, new: Ref) -> Result<()> {
        self.op(name, old, Some(new))
    }

    pub fn create(&mut self, name: BString, new: Ref) -> Result<()> {
        if new.object_id().map_or(false, Hash::is_null) {
            return Err(anyhow!("create called with new object id set to null"));
        }
        self.op(
            name,
            Some(Ref::ObjectId(Hash::null(self.ref_store_inner.hash_kind()))),
            Some(new),
        )
    }

    pub fn delete(&mut self, name: BString, old: Option<Ref>) -> Result<()> {
        if old
            .as_ref()
            .and_then(|old| old.object_id())
            .map_or(false, Hash::is_null)
        {
            return Err(anyhow!("delete called with old object id set to null"));
        }
        self.op(
            name,
            old,
            Some(Ref::ObjectId(Hash::null(self.ref_store_inner.hash_kind()))),
        )
    }

    pub fn verify(&mut self, name: BString, old: Option<Ref>) -> Result<()> {
        self.op(name, old, None)
    }

    fn op(&mut self, name: BString, old: Option<Ref>, new: Option<Ref>) -> Result<()> {
        match self.ops.entry(name) {
            hash_map::Entry::Occupied(_) => Err(anyhow!("")),
            hash_map::Entry::Vacant(entry) => {
                entry.insert(Op { old, new });
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Op {
    old: Option<Ref>,
    new: Option<Ref>,
}
