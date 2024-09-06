use {
    sha1::{Digest, Sha1},
    std::{
        fmt,
        fmt::{Display, Formatter},
        io,
        io::Write,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum Hash {
    Sha1([u8; 20]),
}

impl Hash {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Hash::Sha1(bytes) => bytes,
        }
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Sha1(bytes) => {
                for byte in bytes.iter() {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum Hasher {
    Sha1(Sha1),
}

impl Hasher {
    pub fn new(kind: Kind) -> Self {
        match kind {
            Kind::Sha1 => Hasher::Sha1(Sha1::new()),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        match self {
            Hasher::Sha1(hasher) => hasher.update(data),
        }
    }

    pub fn finalize(self) -> Hash {
        match self {
            Hasher::Sha1(hasher) => Hash::Sha1(hasher.finalize().into()),
        }
    }
}

impl Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Sha1,
}
