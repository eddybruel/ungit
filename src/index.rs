use {
    crate::{
        io::{CountWriter, HashWriter, WriteExt},
        lockfile::Lockfile,
        oid,
        oid::Oid,
        reader::Reader,
    },
    anyhow::{Result, anyhow},
    bitflags::bitflags,
    bstr::BString,
    memmap2::Mmap,
    std::{
        fs,
        io::Write,
        ops::{Deref, DerefMut},
        path::PathBuf,
    },
};

const SIGNATURE: &[u8] = b"DIRC";

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    oid_kind: oid::Kind,
}

impl File {
    pub fn new(path: impl Into<PathBuf>, oid_kind: oid::Kind) -> Self {
        Self::_new(path.into(), oid_kind)
    }

    fn _new(path: PathBuf, oid_kind: oid::Kind) -> Self {
        Self { path, oid_kind }
    }

    pub fn load(&self) -> Result<Index> {
        let file = fs::File::open(&self.path)?;
        let data = unsafe { Mmap::map(&file)? };
        Index::from_bytes(&data, self.oid_kind)
    }

    pub fn load_for_update(&self) -> Result<LoadForUpdate> {
        Ok(LoadForUpdate {
            lockfile: Lockfile::acquire(&self.path)?,
            oid_kind: self.oid_kind,
            index: self.load()?,
        })
    }
}

#[derive(Debug)]
pub struct LoadForUpdate {
    lockfile: Lockfile,
    oid_kind: oid::Kind,
    index: Index,
}

impl LoadForUpdate {
    pub fn commit(mut self) -> Result<()> {
        self.index.write_to(&mut self.lockfile, self.oid_kind)?;
        self.lockfile.commit()
    }
}

impl Deref for LoadForUpdate {
    type Target = Index;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl DerefMut for LoadForUpdate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.index
    }
}

#[derive(Debug)]
pub struct Index {
    version: Version,
    entries: Vec<Entry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            version: Version::V2,
            entries: Vec::new(),
        }
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn from_bytes(bytes: &[u8], oid_kind: oid::Kind) -> Result<Self> {
        let (bytes, expected_checksum) = bytes.split_at(bytes.len() - oid_kind.size());
        let expected_checksum = Oid::from_kind_and_bytes(oid_kind, expected_checksum);
        let mut builder = oid::Builder::new(oid_kind);
        builder.write(bytes);
        let actual_checksum = builder.finish();
        if actual_checksum != expected_checksum {
            return Err(anyhow!("checksum mismatch"));
        }
        let mut reader = Reader::new(bytes);
        let signature = reader.read(4)?;
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
            entries.push(Entry::read_from(&mut reader, oid_kind)?);
        }
        Ok(Self { version, entries })
    }

    pub fn write_to(&self, writer: &mut impl Write, oid_kind: oid::Kind) -> Result<()> {
        let mut writer = HashWriter::new(writer, oid_kind);
        writer.write_all(SIGNATURE)?;
        writer.write_u32(self.version.into())?;
        writer.write_u32(self.entries.len().try_into().unwrap())?;
        for entry in &self.entries {
            entry.write_to(&mut writer)?;
        }
        let (writer, hash) = writer.finish();
        writer.write_all(hash.as_bytes())?;
        Ok(())
    }
}

impl Deref for Index {
    type Target = [Entry];

    fn deref(&self) -> &Self::Target {
        &self.entries
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
    pub oid: Oid,
    pub flags: Flags,
    pub extended_flags: ExtendedFlags,
    pub path: BString,
}

impl Entry {
    pub fn read_from(reader: &mut Reader<'_>, oid_kind: oid::Kind) -> Result<Self> {
        let start = reader.position();
        let stat = Stat::read_from(reader)?;
        let id = Oid::from_kind_and_bytes(oid_kind, reader.read(oid_kind.size())?);
        let flags = Flags::from_bits_retain(reader.read_u16()?);
        let extended_flags = if flags.contains(Flags::EXTENDED) {
            ExtendedFlags::from_bits_retain(reader.read_u16()?)
        } else {
            ExtendedFlags::empty()
        };
        let path = BString::from(if flags.contains(Flags::PATH_LEN) {
            reader.read_until(b'\0')?
        } else {
            let path_len: usize = flags.path_len().try_into().unwrap();
            reader.read(path_len)?
        });
        let num_read = reader.position() - start;
        let padding = 8 - num_read % 8;
        reader.read(padding)?;
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
