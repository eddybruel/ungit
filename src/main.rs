pub mod blob;
pub mod index;
pub mod io;
pub mod lockfile;
pub mod object;
pub mod odb;
pub mod oid;
pub mod reader;

use {anyhow::Result, odb::Odb};

fn main() -> Result<()> {
    let index_file = index::File::new(
        "/Users/ejpbruel/Projects/makepad/.git/index",
        oid::Kind::Sha1,
    );
    let index = index_file.load_for_update()?;
    println!("{:#?}", index);
    index.commit()?;

    let odb = Odb::new("/Users/ejpbruel/Projects/makepad/.git/objects", oid::Kind::Sha1);
    let oid = blob::from_path("/Users/ejpbruel/Projects/makepad/Cargo.toml", &odb)?;
    println!("{}", oid);
    
    Ok(())
}
