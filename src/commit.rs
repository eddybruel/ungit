use {
    crate::{hash::Hash, object, object::Object, object_store::ObjectStore},
    anyhow::Result,
    bstr::BStr,
    jiff::Zoned,
    std::io::Write,
};

#[derive(Clone, Copy, Debug)]
pub struct Signature<'a> {
    pub name: &'a BStr,
    pub email: &'a BStr,
    pub timestamp: Timestamp,
}

impl<'a> Signature<'a> {
    fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(self.name)?;
        writer.write_all(b" <")?;
        writer.write_all(self.email)?;
        writer.write_all(b"> ")?;
        self.timestamp.write_to(writer)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Timestamp {
    pub time: i64,
    pub sign: Sign,
    pub offset: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        let zoned = Zoned::now();
        let offset = zoned.offset();
        Self {
            time: zoned.timestamp().as_second(),
            sign: if offset.signum() >= 0 {
                Sign::Plus
            } else {
                Sign::Minus
            },
            offset: offset.seconds().unsigned_abs(),
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        const SECS_PER_MIN: u32 = 60;
        const MINS_PER_HOUR: u32 = 60;
        const SECS_PER_HOUR: u32 = SECS_PER_MIN * MINS_PER_HOUR;

        let mut buffer = itoa::Buffer::new();
        let hours = self.offset / SECS_PER_HOUR;
        let mins = (self.offset % SECS_PER_HOUR) / SECS_PER_MIN;
        writer.write_all(buffer.format(self.time).as_bytes())?;
        writer.write_all(b" ")?;
        writer.write_all(self.sign.as_bytes())?;
        writer.write_all(buffer.format(hours).as_bytes())?;
        writer.write_all(b":")?;
        writer.write_all(buffer.format(mins).as_bytes())?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn as_bytes(self) -> &'static [u8] {
        match self {
            Sign::Plus => b"+",
            Sign::Minus => b"-",
        }
    }
}

pub fn create(
    tree: Hash,
    parents: &[Hash],
    author: Signature<'_>,
    committer: Signature<'_>,
    message: &BStr,
    store: &ObjectStore,
) -> Result<Hash> {
    let mut content = Vec::new();
    content.write_all(b"tree ")?;
    tree.write_hex_to(&mut content)?;
    content.write_all(b"\n")?;
    for parent in parents {
        content.write_all(b"parent ")?;
        parent.write_hex_to(&mut content)?;
        content.write_all(b"\n")?;
    }
    content.write_all(b"author ")?;
    author.write_to(&mut content)?;
    content.write_all(b"\n")?;
    content.write_all(b"committer ")?;
    committer.write_to(&mut content)?;
    content.write_all(b"\n\n")?;
    content.write_all(message)?;
    store.write(Object {
        kind: object::Kind::Commit,
        content: &content,
    })
}
