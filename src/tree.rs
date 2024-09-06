use {
    crate::{hash::Hash, object, object::Object},
    std::{io, io::Write},
};

#[derive(Clone, Debug)]
pub struct Tree {
    pub entries: Vec<Entry>,
}

impl Object for Tree {
    fn kind(&self) -> object::Kind {
        object::Kind::Tree
    }

    fn write_content_to(&self, writer: &mut impl Write) -> io::Result<()> {
        for entry in &self.entries {
            entry.write_to(writer)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub mode: Mode,
    pub filename: Vec<u8>,
    pub hash: Hash,
}

impl Entry {
    fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        self.mode.write_to(writer)?;
        writer.write_all(b" ")?;
        writer.write_all(&self.filename)?;
        writer.write_all(b"\0")?;
        writer.write_all(self.hash.as_bytes())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Blob,
    Tree,
}

impl Mode {
    fn to_u32(&self) -> u32 {
        match self {
            Mode::Blob => 0o100644,
            Mode::Tree => 0o040000,
        }
    }

    fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        write!(writer, "{:o}", self.to_u32())
    }
}
