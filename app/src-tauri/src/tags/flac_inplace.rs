//! FLAC tag writer that touches only the metadata area.
//!
//! lofty rewrites the whole file on every save; over SMB that is the full file size
//! twice per track. Most rips keep a PADDING block, so a new VORBIS_COMMENT block
//! usually fits: we then rewrite a few KB at the start of the file and never touch
//! the audio frames. Returns Ok(false) when the new block does not fit, so the
//! caller can fall back to a full rewrite.
use crate::model::Fields;
use anyhow::{bail, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::VORBIS_KEYS;

const PADDING: u8 = 1;
const VORBIS: u8 = 4;

#[derive(Debug, Clone)]
struct Block {
    /// offset of the 4-byte block header
    at: u64,
    kind: u8,
    len: usize,
    last: bool,
}

fn header(kind: u8, last: bool, len: usize) -> [u8; 4] {
    let l = len as u32;
    [(if last { 0x80 } else { 0 }) | kind, (l >> 16) as u8, (l >> 8) as u8, l as u8]
}

fn read_blocks(f: &mut File) -> Result<Vec<Block>> {
    let mut head = [0u8; 10];
    f.seek(SeekFrom::Start(0))?;
    f.read_exact(&mut head)?;
    let mut off: u64 = if &head[0..3] == b"ID3" {
        10 + (((head[6] as u64 & 127) << 21) | ((head[7] as u64 & 127) << 14) | ((head[8] as u64 & 127) << 7) | (head[9] as u64 & 127))
    } else {
        0
    };
    let mut magic = [0u8; 4];
    f.seek(SeekFrom::Start(off))?;
    f.read_exact(&mut magic)?;
    if &magic != b"fLaC" {
        bail!("không phải file FLAC hợp lệ");
    }
    off += 4;
    let mut out = vec![];
    loop {
        let mut h = [0u8; 4];
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut h)?;
        let len = ((h[1] as usize) << 16) | ((h[2] as usize) << 8) | h[3] as usize;
        let b = Block { at: off, kind: h[0] & 0x7f, len, last: h[0] & 0x80 != 0 };
        off += 4 + len as u64;
        let last = b.last;
        out.push(b);
        if last || out.len() > 1000 {
            break;
        }
    }
    Ok(out)
}

/// Parsed VORBIS_COMMENT body: vendor string and the comments in file order.
fn parse_vorbis(body: &[u8]) -> Result<(Vec<u8>, Vec<(String, String)>)> {
    let u = |i: usize| -> Result<usize> {
        Ok(u32::from_le_bytes(body.get(i..i + 4).ok_or_else(|| anyhow::anyhow!("vorbis comment lỗi"))?.try_into()?) as usize)
    };
    let vl = u(0)?;
    let vendor = body.get(4..4 + vl).ok_or_else(|| anyhow::anyhow!("vorbis comment lỗi"))?.to_vec();
    let mut q = 4 + vl;
    let n = u(q)?;
    q += 4;
    let mut items = Vec::with_capacity(n);
    for _ in 0..n {
        let l = u(q)?;
        let kv = body.get(q + 4..q + 4 + l).ok_or_else(|| anyhow::anyhow!("vorbis comment lỗi"))?;
        let kv = String::from_utf8_lossy(kv).to_string();
        q += 4 + l;
        let (k, v) = kv.split_once('=').unwrap_or((kv.as_str(), ""));
        items.push((k.to_string(), v.to_string()));
    }
    Ok((vendor, items))
}

fn serialize_vorbis(vendor: &[u8], items: &[(String, String)]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    b.extend_from_slice(vendor);
    b.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for (k, v) in items {
        let kv = format!("{k}={v}");
        b.extend_from_slice(&(kv.len() as u32).to_le_bytes());
        b.extend_from_slice(kv.as_bytes());
    }
    b
}

/// Same key policy as the lofty-based writer: update every spelling the file
/// uses plus the standard key; keep everything else byte for byte.
fn apply_changes(items: &mut Vec<(String, String)>, c: &Fields) {
    for (field, keys) in VORBIS_KEYS {
        let Some(v) = c.get(*field) else { continue };
        let mut targets: Vec<String> = vec![keys[0].to_string()];
        for (k, _) in items.iter() {
            if keys.iter().any(|x| x.eq_ignore_ascii_case(k)) && !targets.iter().any(|t| t.eq_ignore_ascii_case(k)) {
                targets.push(k.clone());
            }
        }
        let pos = items.iter().position(|(k, _)| targets.iter().any(|t| t.eq_ignore_ascii_case(k)));
        items.retain(|(k, _)| !targets.iter().any(|t| t.eq_ignore_ascii_case(k)));
        if !v.is_empty() {
            let at = pos.unwrap_or(items.len()).min(items.len());
            for (i, t) in targets.into_iter().enumerate() {
                items.insert(at + i, (t, v.clone()));
            }
        }
    }
    // a "3/12" style number would contradict a new total: keep the plain number
    for (n, tot, key) in [("track", "tracktotal", "TRACKNUMBER"), ("disc", "disctotal", "DISCNUMBER")] {
        if c.contains_key(tot) && !c.contains_key(n) {
            for (k, v) in items.iter_mut() {
                if k.eq_ignore_ascii_case(key) && v.contains('/') {
                    *v = v.split('/').next().unwrap_or("").trim().to_string();
                }
            }
        }
    }
}

/// Try the metadata-only write. Ok(true): done in place. Ok(false): did not fit,
/// nothing was written.
pub fn write(p: &Path, changes: &Fields) -> Result<bool> {
    let mut f = OpenOptions::new().read(true).write(true).open(p)?;
    let blocks = read_blocks(&mut f)?;
    let Some(vi) = blocks.iter().position(|b| b.kind == VORBIS) else { return Ok(false) };
    let v = blocks[vi].clone();
    let mut body = vec![0u8; v.len];
    f.seek(SeekFrom::Start(v.at + 4))?;
    f.read_exact(&mut body)?;
    let (vendor, mut items) = parse_vorbis(&body)?;
    apply_changes(&mut items, changes);
    let new = serialize_vorbis(&vendor, &items);
    let n = new.len();
    if n > 0xFF_FFFF {
        return Ok(false);
    }

    let mut out: Vec<(u64, Vec<u8>)> = vec![]; // (offset, bytes) to write
    let next = blocks.get(vi + 1).filter(|b| b.kind == PADDING && b.at == v.at + 4 + v.len as u64);
    if n == v.len {
        // 1. same size: overwrite
        let mut w = header(VORBIS, v.last, n).to_vec();
        w.extend_from_slice(&new);
        out.push((v.at, w));
    } else if let Some(pad) = next.filter(|pad| n <= v.len + pad.len) {
        // 2. grow/shrink into the padding right after it
        let rest = v.len + pad.len - n; // payload of the new padding block
        let mut w = header(VORBIS, false, n).to_vec();
        w.extend_from_slice(&new);
        w.extend_from_slice(&header(PADDING, pad.last, rest));
        w.extend(std::iter::repeat(0u8).take(rest));
        out.push((v.at, w));
    } else if let Some(pad) = next.filter(|pad| n == v.len + 4 + pad.len) {
        // 2b. exactly fills comments + padding (header included): padding block disappears
        let mut w = header(VORBIS, pad.last, n).to_vec();
        w.extend_from_slice(&new);
        out.push((v.at, w));
    } else if n + 4 <= v.len {
        // 3. shrink in its own slot, leftover becomes padding
        let rest = v.len - n - 4;
        let mut w = header(VORBIS, false, n).to_vec();
        w.extend_from_slice(&new);
        w.extend_from_slice(&header(PADDING, v.last, rest));
        w.extend(std::iter::repeat(0u8).take(rest));
        out.push((v.at, w));
    } else if let Some(pad) = blocks.iter().find(|b| b.kind == PADDING && (b.len == n || b.len >= n + 4)) {
        // 4. move it into a padding block elsewhere; its old slot becomes padding
        let mut old = header(PADDING, v.last, v.len).to_vec();
        old.extend(std::iter::repeat(0u8).take(v.len));
        out.push((v.at, old));
        let mut w = header(VORBIS, pad.last && pad.len == n, n).to_vec();
        w.extend_from_slice(&new);
        if pad.len != n {
            let rest = pad.len - n - 4;
            w.extend_from_slice(&header(PADDING, pad.last, rest));
            w.extend(std::iter::repeat(0u8).take(rest));
        }
        out.push((pad.at, w));
    } else {
        return Ok(false);
    }
    for (at, bytes) in out {
        f.seek(SeekFrom::Start(at))?;
        f.write_all(&bytes)?;
    }
    f.sync_all()?;
    Ok(true)
}
