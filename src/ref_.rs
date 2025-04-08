use {
    crate::{hash, hash::Hash, reader::Reader},
    anyhow::Result,
    bstr::BString,
    std::io::Write,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ref {
    ObjectId(Hash),
    RefName(BString),
}

impl Ref {
    pub fn from_bytes(bytes: &[u8], hash_kind: hash::Kind) -> Result<Self> {
        let bytes = if bytes.ends_with(b"\n") {
            &bytes[..bytes.len() - 1]
        } else {
            bytes
        };
        let mut reader = Reader::new(bytes);
        if reader.skip_bytes_if_matches(b"ref: ") {
            Ok(Ref::RefName(reader.read_bytes_until_end().into()))
        } else {
            Ok(Ref::ObjectId(Hash::read_hex_from(&mut reader, hash_kind)?))
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            Ref::ObjectId(object_id) => object_id.write_hex_to(writer),
            Ref::RefName(ref_name) => {
                writer.write_all(b"ref: ")?;
                writer.write_all(ref_name)?;
                Ok(())
            }
        }
    }
}
