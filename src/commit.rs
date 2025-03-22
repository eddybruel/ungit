use {
    crate::{object, object::Object, odb::Odb, oid::Oid},
    anyhow::Result,
    bstr::BStr,
    jiff::Zoned,
    std::{fmt, io::Write},
};

#[derive(Clone, Copy, Debug)]
pub struct Signature<'a> {
    pub name: &'a BStr,
    pub email: &'a BStr,
    pub timestamp: Timestamp,
}

impl fmt::Display for Signature<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <{}> {}", self.name, self.email, self.timestamp)
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
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const SECS_PER_MIN: u32 = 60;
        const MINS_PER_HOUR: u32 = 60;
        const SECS_PER_HOUR: u32 = SECS_PER_MIN * MINS_PER_HOUR;

        let hours = self.offset / SECS_PER_HOUR;
        let mins = (self.offset % SECS_PER_HOUR) / SECS_PER_MIN;
        write!(f, "{} {}{:02}{:02}", self.time, self.sign, hours, mins)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn as_str(self) -> &'static str {
        match self {
            Sign::Plus => "+",
            Sign::Minus => "-",
        }
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn create(
    tree: Oid,
    parents: &[Oid],
    author: Signature<'_>,
    committer: Signature<'_>,
    message: &BStr,
    odb: &Odb,
) -> Result<Oid> {
    let mut content = Vec::new();
    writeln!(&mut content, "tree {}", tree)?;
    for parent in parents {
        writeln!(&mut content, "parent {}", parent)?;
    }
    writeln!(&mut content, "author {}", author)?;
    writeln!(&mut content, "committer {}", committer)?;
    writeln!(&mut content, "")?;
    content.write_all(message)?;
    odb.store(Object {
        kind: object::Kind::Commit,
        content: &content,
    })
}
