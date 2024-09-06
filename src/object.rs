use {
    crate::io::CountWriter,
    std::{
        fmt,
        fmt::{Display, Formatter},
        io,
        io::Write,
    },
};

pub trait Object {
    fn kind(&self) -> Kind;

    fn size(&self) -> usize {
        let mut writer = CountWriter::new(io::sink());
        self.write_content_to(&mut writer).unwrap();
        writer.count()
    }

    fn write_header_to(&self, writer: &mut impl Write) -> io::Result<()> {
        write!(writer, "{} {}\0", self.kind(), self.size())
    }

    fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        self.write_header_to(writer)?;
        self.write_content_to(writer)
    }

    fn write_content_to(&self, writer: &mut impl Write) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl Kind {
    fn as_str(&self) -> &str {
        match self {
            Kind::Blob => "blob",
            Kind::Tree => "tree",
            Kind::Commit => "commit",
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
