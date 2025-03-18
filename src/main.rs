pub mod hash;
pub mod index;
pub mod io;
pub mod lockfile;
pub mod reader;

use anyhow::Result;

fn main() -> Result<()> {
    let file = index::File::new(
        "/Users/ejpbruel/Projects/makepad/.git/index",
        hash::Kind::Sha1,
    );
    let index = file.load_for_update()?;
    println!("{:#?}", index);
    index.commit()?;
    Ok(())
}