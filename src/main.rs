pub mod blob;
pub mod commit;
pub mod index;
pub mod io;
pub mod lockfile;
pub mod object;
pub mod odb;
pub mod oid;
pub mod reader;
pub mod tree;

use {anyhow::Result, commit::Timestamp, odb::Odb};

fn main() -> Result<()> {
    let odb = Odb::new(
        "/Users/ejpbruel/Projects/makepad/.git/objects",
        oid::Kind::Sha1,
    );
    let index_file = index::File::new(
        "/Users/ejpbruel/Projects/makepad/.git/index",
        oid::Kind::Sha1,
    );
    let index = index_file.load()?;
    let tree = tree::from_index(&index, &odb)?;
    let oid = commit::create(
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
        &odb,
    )?;
    println!("{}", oid);
    Ok(())
}
