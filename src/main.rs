pub mod blob;
pub mod index;
pub mod io;
pub mod lockfile;
pub mod object;
pub mod odb;
pub mod oid;
pub mod reader;
pub mod tree;

use {anyhow::Result, odb::Odb};

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
    let oid = tree::from_index(&index, &odb)?;
    println!("{}", oid);
    Ok(())
}
