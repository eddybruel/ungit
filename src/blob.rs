use {
    crate::{object, object::Object},
    std::{io, io::Write},
};

#[derive(Clone, Debug)]
pub struct Blob {
    pub data: Vec<u8>,
}

impl Object for Blob {
    fn kind(&self) -> object::Kind {
        object::Kind::Blob
    }

    fn write_content_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.data)
    }
}
