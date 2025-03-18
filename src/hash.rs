use {digest::Digest, sha1::Sha1, std::fmt};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Hash {
    Sha1([u8; 20]),
}

impl Hash {
    pub fn from_kind_and_bytes(kind: Kind, bytes: &[u8]) -> Self {
        match kind {
            Kind::Sha1 => Hash::Sha1(bytes.try_into().unwrap()),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Hash::Sha1(bytes) => bytes,
        }
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &byte in self.as_bytes() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
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

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Sha1,
}

impl Kind {
    pub fn size(self) -> usize {
        match self {
            Kind::Sha1 => 20,
        }
    }
}
