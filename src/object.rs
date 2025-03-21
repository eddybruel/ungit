use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Object<'a> {
    pub kind: Kind,
    pub content: &'a [u8],
}

impl<'a> Object<'a> {
    pub fn header(&self) -> Header {
        Header {
            kind: self.kind,
            len: self.content.len().try_into().unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub kind: Kind,
    pub len: u64,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}\0", self.kind, self.len)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Blob => "blob",
            Kind::Tree => "tree",
            Kind::Commit => "commit",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
