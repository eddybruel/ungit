use {
    crate::{index::Index, object, object::Object, odb::Odb, oid::Oid},
    anyhow::Result,
    bstr::{BStr, ByteSlice},
    std::io::Write,
};

#[derive(Debug)]
pub struct Writer {
    content: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn write_entry(&mut self, entry: Entry<'_>) -> Result<()> {
        entry.write_to(&mut self.content)
    }

    pub fn finish(self, odb: &Odb) -> Result<Oid> {
        odb.store(Object {
            kind: object::Kind::Tree,
            content: &self.content,
        })
    }
}

#[derive(Debug)]
pub struct Entry<'a> {
    pub mode: Mode,
    pub file_name: &'a BStr,
    pub oid: Oid,
}

impl<'a> Entry<'a> {
    pub fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        write!(writer, "{:o} ", u16::from(self.mode))?;
        writer.write_all(self.file_name.as_bytes())?;
        writer.write_all(b"\0")?;
        writer.write_all(self.oid.as_bytes())?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mode(u16);

impl From<Mode> for u16 {
    fn from(mode: Mode) -> u16 {
        mode.0
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum EntryKind {
    Blob = 0o100644,
    Tree = 0o040000,
}

impl From<EntryKind> for Mode {
    fn from(kind: EntryKind) -> Mode {
        Mode(kind as u16)
    }
}

pub fn from_index(index: &Index, odb: &Odb) -> Result<Oid> {
    let mut entry_idx = 0;
    from_index_recursive(index, odb, b"".into(), &mut entry_idx)
}

fn from_index_recursive(
    index: &Index,
    odb: &Odb,
    dir_name: &BStr,
    entry_idx: &mut usize,
) -> Result<Oid> {
    let mut writer = Writer::new();
    while *entry_idx < index.entries().len() {
        let entry = &index.entries()[*entry_idx];
        if !entry.path.starts_with(dir_name) {
            break;
        }
        let file_name_start = if dir_name.is_empty() {
            dir_name.len()
        } else if entry.path[dir_name.len()] == b'/' {
            dir_name.len() + 1
        } else {
            break;
        };
        writer.write_entry(
            if let Some(file_name_end) = entry.path[file_name_start..].find_byte(b'/') {
                let file_name_end = file_name_start + file_name_end;
                Entry {
                    mode: EntryKind::Tree.into(),
                    file_name: entry.path[file_name_start..file_name_end].into(),
                    oid: from_index_recursive(
                        index,
                        odb,
                        entry.path[..file_name_end].into(),
                        entry_idx,
                    )?,
                }
            } else {
                Entry {
                    mode: EntryKind::Blob.into(),
                    file_name: entry.path[file_name_start..].into(),
                    oid: entry.oid,
                }
            },
        )?;
        *entry_idx += 1;
    }
    writer.finish(odb)
}
