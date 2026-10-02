//! Locate the audio payload of a file so a write can be checked to leave it untouched.
use crate::dsf;
use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

fn ext(path: &Path) -> String {
    path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

/// Returns (offset, length) of the audio data.
pub fn range(path: &Path) -> Result<(u64, u64)> {
    let mut f = File::open(path)?;
    let len = f.metadata()?.len();
    match ext(path).as_str() {
        "flac" => {
            let mut off = 0u64;
            let mut h = [0u8; 4];
            f.read_exact(&mut h)?;
            if &h == b"ID3" as &[u8] || &h[..3] == b"ID3" {
                let mut r = [0u8; 6];
                f.read_exact(&mut r)?;
                let sz = ((r[2] as u64 & 127) << 21) | ((r[3] as u64 & 127) << 14) | ((r[4] as u64 & 127) << 7) | (r[5] as u64 & 127);
                off = 10 + sz;
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut h)?;
            }
            if &h != b"fLaC" {
                bail!("không thấy fLaC");
            }
            off += 4;
            loop {
                let mut b = [0u8; 4];
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut b)?;
                let n = ((b[1] as u64) << 16) | ((b[2] as u64) << 8) | b[3] as u64;
                off += 4 + n;
                if b[0] & 0x80 != 0 {
                    break;
                }
            }
            Ok((off, len - off))
        }
        "wav" | "aif" | "aiff" => {
            let mut h = [0u8; 12];
            f.read_exact(&mut h)?;
            let be = &h[0..4] == b"FORM";
            let want: &[u8; 4] = if be { b"SSND" } else { b"data" };
            let mut off = 12u64;
            while off + 8 <= len {
                let mut c = [0u8; 8];
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut c)?;
                let sz = if be { u32::from_be_bytes(c[4..8].try_into()?) } else { u32::from_le_bytes(c[4..8].try_into()?) } as u64;
                if &c[0..4] == want {
                    return Ok((off + 8, sz));
                }
                off += 8 + sz + (sz & 1);
            }
            bail!("không thấy chunk âm thanh")
        }
        "dsf" => dsf::audio_range(path),
        "m4a" | "mp4" => {
            let mut off = 0u64;
            while off + 8 <= len {
                let mut c = [0u8; 16];
                f.seek(SeekFrom::Start(off))?;
                f.read_exact(&mut c[..8])?;
                let mut sz = u32::from_be_bytes(c[0..4].try_into()?) as u64;
                let mut hdr = 8;
                if sz == 1 {
                    f.read_exact(&mut c[8..16])?;
                    sz = u64::from_be_bytes(c[8..16].try_into()?);
                    hdr = 16;
                }
                if &c[4..8] == b"mdat" {
                    return Ok((off + hdr, sz - hdr));
                }
                if sz == 0 {
                    break;
                }
                off += sz;
            }
            bail!("không thấy mdat")
        }
        "ape" => {
            // audio = whole file minus trailing ID3v1 and APEv2 tag
            let mut end = len;
            let mut t = [0u8; 3];
            if len > 128 {
                f.seek(SeekFrom::Start(len - 128))?;
                f.read_exact(&mut t)?;
                if &t == b"TAG" {
                    end -= 128;
                }
            }
            if end > 32 {
                let mut ft = [0u8; 32];
                f.seek(SeekFrom::Start(end - 32))?;
                f.read_exact(&mut ft)?;
                if &ft[0..8] == b"APETAGEX" {
                    let size = u32::from_le_bytes(ft[12..16].try_into()?) as u64;
                    let flags = u32::from_le_bytes(ft[20..24].try_into()?);
                    end -= size + if flags & 0x8000_0000 != 0 { 32 } else { 0 };
                }
            }
            Ok((0, end))
        }
        e => bail!("chưa hỗ trợ băm audio cho .{e}"),
    }
}

pub fn hash(path: &Path) -> Result<String> {
    let (off, n) = range(path)?;
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(off))?;
    let mut h = Sha256::new();
    let mut left = n;
    let mut buf = vec![0u8; 1 << 20];
    while left > 0 {
        let k = left.min(buf.len() as u64) as usize;
        f.read_exact(&mut buf[..k])?;
        h.update(&buf[..k]);
        left -= k as u64;
    }
    Ok(format!("{:x}", h.finalize())[..16].to_string())
}
