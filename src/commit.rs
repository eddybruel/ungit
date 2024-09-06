use {
    crate::{
        hash::Hash,
        object::{self, Object},
    },
    std::{io, io::Write},
};

#[derive(Clone, Debug)]
pub struct Commit {
    pub tree: Hash,
    pub author: Signature,
    pub committer: Signature,
    pub message: Vec<u8>,
}

impl Object for Commit {
    fn kind(&self) -> object::Kind {
        object::Kind::Commit
    }

    fn write_content_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(b"tree")?;
        writer.write_all(b" ")?;
        writer.write_all(self.tree.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.write_all(b"author")?;
        writer.write_all(b" ")?;
        self.author.write_to(writer)?;
        writer.write_all(b"\n")?;
        writer.write_all(b"committer")?;
        writer.write_all(b" ")?;
        self.committer.write_to(writer)?;
        writer.write_all(b"\n")?;
        writer.write_all(b"\n")?;
        writer.write_all(&self.message)
    }
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub name: Vec<u8>,
    pub email: Vec<u8>,
    pub time: Time,
}

impl Signature {
    fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.name)?;
        writer.write_all(b" ")?;
        writer.write_all(b"<")?;
        writer.write_all(&self.email)?;
        writer.write_all(b">")?;
        writer.write_all(b" ")?;
        self.time.write_to(writer)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Time {
    pub unix_time: i64,
    pub utc_offset: i16,
}

impl Time {
    fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        const MINS_PER_HOUR: u16 = 60;

        let sign = if self.utc_offset.is_negative() {
            "-"
        } else {
            "+"
        };
        let offset = self.utc_offset.unsigned_abs();
        let hours = offset / MINS_PER_HOUR;
        let mins = offset % MINS_PER_HOUR;
        write!(writer, "{} {}{:02}{:02}", self.unix_time, sign, hours, mins,)
    }
}
