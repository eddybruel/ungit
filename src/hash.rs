use {
    crate::reader::Reader,
    anyhow::Result,
    digest::Digest,
    sha1::Sha1,
    std::{fmt, io::Write, str},
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Hash {
    Sha1([u8; 20]),
}

impl Hash {
    pub fn new(kind: Kind) -> Self {
        match kind {
            Kind::Sha1 => Hash::Sha1([0; 20]),
        }
    }

    pub fn from_bytes(bytes: &[u8], kind: Kind) -> Self {
        match kind {
            Kind::Sha1 => Hash::Sha1(bytes.try_into().unwrap()),
        }
    }

    pub fn from_hex_bytes(hex_bytes: &[u8], kind: Kind) -> Result<Self> {
        assert_eq!(hex_bytes.len(), kind.hex_len());
        let mut buffer = [0; Kind::LONGEST.len()];
        let bytes = &mut buffer[..kind.len()];
        if hex::decode_to_slice(hex_bytes, bytes).is_err() {
            return Err(anyhow::anyhow!("invalid hex byte"));
        }
        Ok(Self::from_bytes(bytes, kind))
    }

    pub fn read_from(reader: &mut Reader, kind: Kind) -> Result<Self> {
        Ok(Self::from_bytes(reader.read_bytes(kind.len())?, kind))
    }

    pub fn read_hex_from(reader: &mut Reader, kind: Kind) -> Result<Self> {
        Ok(Self::from_hex_bytes(
            reader.read_bytes(kind.hex_len())?,
            kind,
        )?)
    }

    pub fn kind(&self) -> Kind {
        match self {
            Hash::Sha1(_) => Kind::Sha1,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Hash::Sha1(bytes) => bytes,
        }
    }

    pub fn to_hex_str<'a>(&self, buffer: &'a mut [u8]) -> &'a str {
        let hex_bytes = &mut buffer[..self.kind().hex_len()];
        hex::encode_to_slice(self.as_bytes(), hex_bytes).unwrap();
        str::from_utf8(hex_bytes).unwrap()
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(self.as_bytes())?;
        Ok(())
    }

    pub fn write_hex_to(&self, writer: &mut impl Write) -> Result<()> {
        let mut hex_bytes = [0; Kind::LONGEST.hex_len()];
        writer.write_all(self.to_hex_str(&mut hex_bytes).as_bytes())?;
        Ok(())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0; Kind::LONGEST.hex_len()];
        f.write_str(self.to_hex_str(&mut buffer))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0; Kind::LONGEST.hex_len()];
        f.write_str(self.to_hex_str(&mut buffer))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Sha1,
}

impl Kind {
    pub const LONGEST: Self = Self::Sha1;

    pub const fn len(self) -> usize {
        match self {
            Kind::Sha1 => 20,
        }
    }

    pub const fn hex_len(self) -> usize {
        self.len() * 2
    }
}

#[derive(Clone, Debug)]
pub enum Builder {
    Sha1(Sha1),
}

impl Builder {
    pub fn new(kind: Kind) -> Self {
        match kind {
            Kind::Sha1 => Builder::Sha1(Sha1::new()),
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        match self {
            Builder::Sha1(hasher) => hasher.update(data),
        }
    }

    pub fn finish(self) -> Hash {
        match self {
            Builder::Sha1(hasher) => Hash::Sha1(hasher.finalize().try_into().unwrap()),
        }
    }
}
