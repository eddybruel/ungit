pub mod blob;
pub mod commit;
pub mod hash;
pub mod index;
pub mod index_file;
pub mod io;
pub mod lock_file;
pub mod object;
pub mod object_store;
pub mod reader;
pub mod ref_;
pub mod ref_store;
pub mod tree;

use {anyhow::Result, commit::Timestamp, index_file::IndexFile, object_store::ObjectStore};

fn main() -> Result<()> {
    let object_store = ObjectStore::new(
        "/Users/ejpbruel/Projects/makepad/.git/objects",
        hash::Kind::Sha1,
    );
    let index_file = IndexFile::new(
        "/Users/ejpbruel/Projects/makepad/.git/index",
        hash::Kind::Sha1,
    );
    let index = index_file.read()?;
    let tree = tree::from_index(&index, &object_store)?;
    let commit_id = commit::create(
        tree,
        &[],
        commit::Signature {
            name: b"Eddy Bruel".into(),
            email: b"me@eddybruel".into(),
            timestamp: Timestamp::now(),
        },
        commit::Signature {
            name: b"Eddy Bruel".into(),
            email: b"me@eddybruel".into(),
            timestamp: Timestamp::now(),
        },
        b"Initial commit".into(),
        &object_store,
    )?;
    println!("oid: {}", commit_id);
    Ok(())
}
