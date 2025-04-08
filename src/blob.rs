use {
    crate::{hash::Hash, object, object_store::ObjectStore},
    anyhow::Result,
    std::{fs::File, io, io::BufReader, path::Path},
};

pub fn from_path(path: impl AsRef<Path>, store: &ObjectStore) -> Result<Hash> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut writer = store.create_writer(object::Header {
        kind: object::Kind::Blob,
        len,
    })?;
    io::copy(&mut reader, &mut writer)?;
    writer.finish()
}
