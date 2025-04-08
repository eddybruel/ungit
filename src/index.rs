use {
    crate::{
        hash,
        hash::Hash,
        io::{CountWriter, HashWriter, WriteExt},
        reader::Reader,
    },
    anyhow::{Result, anyhow},
    bitflags::bitflags,
    bstr::BString,
    std::io::Write,
};

const SIGNATURE: &[u8] = b"DIRC";

#[derive(Debug)]
pub struct Index {
    version: Version,
    entries: Vec<Entry>,
}

impl Index {
    pub fn from_bytes(bytes: &[u8], hash_kind: hash::Kind) -> Result<Self> {
        let (bytes, expected_checksum) = bytes.split_at(bytes.len() - hash_kind.len());
        let expected_checksum = Hash::from_bytes(expected_checksum, hash_kind);
        let mut builder = hash::Builder::new(hash_kind);
        builder.write(bytes);
        let actual_checksum = builder.finish();
        if actual_checksum != expected_checksum {
            return Err(anyhow!("checksum mismatch"));
        }
        let mut reader = Reader::new(bytes);
        let signature = reader.read_bytes(4)?;
        if signature != SIGNATURE {
            return Err(anyhow!("invalid signature"));
        }
        let version = match reader.read_u32()? {
            2 => Version::V2,
            3 => Version::V3,
            _ => return Err(anyhow!("unsupported version")),
        };
        let num_entries = reader.read_u32()?.try_into().unwrap();
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            entries.push(Entry::read_from(&mut reader, hash_kind)?);
        }
        Ok(Self { version, entries })
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn write_to(&self, writer: &mut impl Write, hash_kind: hash::Kind) -> Result<()> {
        let mut writer = HashWriter::new(writer, hash_kind);
        writer.write_all(SIGNATURE)?;
        writer.write_u32(self.version.into())?;
        writer.write_u32(self.entries.len().try_into().unwrap())?;
        for entry in &self.entries {
            entry.write_to(&mut writer)?;
        }
        let (writer, checksum) = writer.finish();
        writer.write_all(checksum.as_bytes())?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum Version {
    V2 = 2,
    V3 = 3,
}

impl From<Version> for u32 {
    fn from(version: Version) -> u32 {
        version as u32
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub stat: Stat,
    pub oid: Hash,
    pub flags: Flags,
    pub extended_flags: ExtendedFlags,
    pub path: BString,
}

impl Entry {
    pub fn read_from(reader: &mut Reader<'_>, hash_kind: hash::Kind) -> Result<Self> {
        let start = reader.position();
        let stat = Stat::read_from(reader)?;
        let id = Hash::from_bytes(reader.read_bytes(hash_kind.len())?, hash_kind);
        let flags = Flags::from_bits_retain(reader.read_u16()?);
        let extended_flags = if flags.contains(Flags::EXTENDED) {
            ExtendedFlags::from_bits_retain(reader.read_u16()?)
        } else {
            ExtendedFlags::empty()
        };
        let path = BString::from(if flags.contains(Flags::PATH_LEN) {
            reader.read_bytes_until_found(b'\0')?
        } else {
            let path_len: usize = flags.path_len().try_into().unwrap();
            reader.read_bytes(path_len)?
        });
        let num_read = reader.position() - start;
        let padding = 8 - num_read % 8;
        reader.read_bytes(padding)?;
        Ok(Self {
            stat,
            oid: id,
            flags,
            extended_flags,
            path,
        })
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        let mut writer = CountWriter::new(writer);
        self.stat.write_to(&mut writer)?;
        writer.write_all(&self.oid.as_bytes())?;
        writer.write_u16(self.flags.bits())?;
        if self.flags.contains(Flags::EXTENDED) {
            writer.write_u16(self.extended_flags.bits())?;
        }
        writer.write_all(&self.path)?;
        let num_written = writer.count();
        let writer = writer.into_inner();
        let padding = 8 - num_written % 8;
        let zeroes = [0; 8];
        writer.write_all(&zeroes[..padding])?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Stat {
    pub ctime: Time,
    pub mtime: Time,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
}

impl Stat {
    pub fn read_from(reader: &mut Reader<'_>) -> Result<Self> {
        Ok(Self {
            ctime: Time::read_from(reader)?,
            mtime: Time::read_from(reader)?,
            dev: reader.read_u32()?,
            ino: reader.read_u32()?,
            mode: reader.read_u32()?,
            uid: reader.read_u32()?,
            gid: reader.read_u32()?,
            size: reader.read_u32()?,
        })
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        self.ctime.write_to(writer)?;
        self.mtime.write_to(writer)?;
        writer.write_u32(self.dev)?;
        writer.write_u32(self.ino)?;
        writer.write_u32(self.mode)?;
        writer.write_u32(self.uid)?;
        writer.write_u32(self.gid)?;
        writer.write_u32(self.size)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Time {
    pub secs: u32,
    pub nsecs: u32,
}

impl Time {
    pub fn read_from(reader: &mut Reader<'_>) -> Result<Self> {
        Ok(Self {
            secs: reader.read_u32()?,
            nsecs: reader.read_u32()?,
        })
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_u32(self.secs)?;
        writer.write_u32(self.nsecs)?;
        Ok(())
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Flags: u16 {
        const ASSUME_VALID = 0x8000;
        const EXTENDED = 0x4000;
        const STAGE = 0x3000;
        const PATH_LEN = 0x0FFF;
    }
}

impl Flags {
    pub fn path_len(self) -> u16 {
        (self & Self::PATH_LEN).bits()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ExtendedFlags: u16 {
        const SKIP_WORKTREE = 0x4000;
        const INTENT_TO_ADD = 0x2000;
    }
}
