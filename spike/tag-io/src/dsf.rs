//! Minimal DSF tag support (lofty has none).
//!
//! Layout: "DSD " chunk (28 bytes: id, chunk size u64 = 28, total file size u64,
//! metadata pointer u64), then "fmt " chunk, then "data" chunk. The tag is an
//! ID3v2 block that starts at the metadata pointer and runs to EOF.
use anyhow::{bail, Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct Layout {
    pub file_size: u64,
    pub meta_ptr: u64,
    pub data_off: u64,
    pub data_len: u64,
}

fn u64le(b: &[u8]) -> u64 {
    u64::from_le_bytes(b[..8].try_into().unwrap())
}

pub fn layout(f: &mut File) -> Result<Layout> {
    let mut h = [0u8; 28];
    f.seek(SeekFrom::Start(0))?;
    f.read_exact(&mut h)?;
    if &h[0..4] != b"DSD " {
        bail!("không phải file DSF");
    }
    let file_size = u64le(&h[12..]);
    let meta_ptr = u64le(&h[20..]);
    let mut off = u64le(&h[4..]); // = 28
    loop {
        let mut ch = [0u8; 12];
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut ch).context("thiếu chunk data")?;
        let size = u64le(&ch[4..]);
        if &ch[0..4] == b"data" {
            return Ok(Layout { file_size, meta_ptr, data_off: off + 12, data_len: size - 12 });
        }
        if size == 0 {
            bail!("chunk lỗi");
        }
        off += size;
    }
}

pub fn read_tag(path: &Path) -> Result<Option<id3::Tag>> {
    let mut f = File::open(path)?;
    let l = layout(&mut f)?;
    if l.meta_ptr == 0 {
        return Ok(None);
    }
    f.seek(SeekFrom::Start(l.meta_ptr))?;
    match id3::Tag::read_from2(&mut f) {
        Ok(t) => Ok(Some(t)),
        Err(e) if matches!(e.kind, id3::ErrorKind::NoTag) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Replace the ID3v2 block at the end of the file and fix the header fields.
pub fn write_tag(path: &Path, tag: &id3::Tag) -> Result<()> {
    let mut f = OpenOptions::new().read(true).write(true).open(path)?;
    let l = layout(&mut f)?;
    let data_end = l.data_off + l.data_len;
    let start = if l.meta_ptr != 0 { l.meta_ptr.max(data_end) } else { data_end };
    let mut buf = Vec::new();
    let version = if tag.version() == id3::Version::Id3v22 { id3::Version::Id3v23 } else { tag.version() };
    tag.write_to(&mut buf, version)?;
    f.set_len(start)?;
    f.seek(SeekFrom::Start(start))?;
    f.write_all(&buf)?;
    let total = start + buf.len() as u64;
    f.seek(SeekFrom::Start(12))?;
    f.write_all(&total.to_le_bytes())?;
    f.write_all(&start.to_le_bytes())?;
    f.sync_all()?;
    Ok(())
}

pub fn audio_range(path: &Path) -> Result<(u64, u64)> {
    let mut f = File::open(path)?;
    let l = layout(&mut f)?;
    Ok((l.data_off, l.data_len))
}
