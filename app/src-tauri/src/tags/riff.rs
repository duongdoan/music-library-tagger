//! RIFF INFO writer that keeps the raw bytes of every item it does not change.
//! lofty drops INFO values that are not valid UTF-8 (legacy Windows code pages).
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

fn le32(b: &[u8]) -> usize {
    u32::from_le_bytes(b[..4].try_into().unwrap()) as usize
}

fn info_body(items: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
    let mut b = b"INFO".to_vec();
    for (id, v) in items {
        let mut val = v.clone();
        val.push(0); // NUL terminated
        b.extend_from_slice(id);
        b.extend_from_slice(&(val.len() as u32).to_le_bytes());
        b.extend_from_slice(&val);
        if val.len() % 2 == 1 {
            b.push(0);
        }
    }
    b
}

fn valid_id(id: &[u8]) -> bool {
    id.iter().all(|b| b.is_ascii_alphanumeric() || *b == b' ')
}

/// Rewrite a WAV in one pass: replace the LIST/INFO items in `updates` (an empty
/// value removes the item) and, when `id3` is given, replace or add the ID3 chunk.
/// Everything else keeps its bytes. Bytes after the last well-formed chunk (some
/// taggers leave a non-RIFF trailer) are kept at the end, after our chunks, so
/// chunk walkers still reach the tags.
pub fn write_wav(p: &Path, updates: &[(&str, &str)], id3: Option<&[u8]>) -> Result<()> {
    let data = fs::read(p)?;
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        bail!("không phải WAV");
    }
    let mut chunks: Vec<(usize, usize)> = vec![];
    let mut off = 12;
    let mut trailer = data.len();
    while off + 8 <= data.len() {
        let sz = le32(&data[off + 4..]);
        let end = off + 8 + sz + (sz & 1);
        if !valid_id(&data[off..off + 4]) || off + 8 + sz > data.len() {
            trailer = off;
            break;
        }
        chunks.push((off, end.min(data.len())));
        off = end;
    }
    if off < data.len() && trailer == data.len() && off + 8 > data.len() {
        trailer = off;
    }
    let is_info = |(s, _): &(usize, usize)| &data[*s..*s + 4] == b"LIST" && data.len() >= *s + 12 && &data[*s + 8..*s + 12] == b"INFO";
    let is_id3 = |(s, _): &(usize, usize)| data[*s..*s + 4].eq_ignore_ascii_case(b"id3 ");
    let mut items: Vec<([u8; 4], Vec<u8>)> = vec![];
    if let Some(&(s, e)) = chunks.iter().find(|c| is_info(c)) {
        let body = &data[s + 12..(s + 8 + le32(&data[s + 4..])).min(e)];
        let mut q = 0;
        while q + 8 <= body.len() {
            let n = le32(&body[q + 4..]);
            let raw = &body[q + 8..(q + 8 + n).min(body.len())];
            let v = raw.split(|b| *b == 0).next().unwrap_or(&[]).to_vec();
            items.push((body[q..q + 4].try_into().unwrap(), v));
            q += 8 + n + (n & 1);
        }
    }
    for (id, val) in updates {
        let id: [u8; 4] = id.as_bytes().try_into()?;
        let pos = items.iter().position(|(k, _)| *k == id);
        items.retain(|(k, _)| *k != id);
        if !val.is_empty() {
            let at = pos.unwrap_or(items.len()).min(items.len());
            items.insert(at, (id, val.as_bytes().to_vec()));
        }
    }
    let chunk = |id: &[u8], body: &[u8]| -> Vec<u8> {
        let mut c = id.to_vec();
        c.extend_from_slice(&(body.len() as u32).to_le_bytes());
        c.extend_from_slice(body);
        if body.len() % 2 == 1 {
            c.push(0);
        }
        c
    };
    let list = if items.is_empty() { vec![] } else { chunk(b"LIST", &info_body(&items)) };
    let id3_id: Vec<u8> = chunks.iter().find(|c| is_id3(c)).map(|&(s, _)| data[s..s + 4].to_vec()).unwrap_or_else(|| b"ID3 ".to_vec());

    let mut out = data[0..12].to_vec();
    let (mut info_done, mut id3_done) = (false, id3.is_none());
    for c in &chunks {
        if is_info(c) {
            if !info_done {
                out.extend_from_slice(&list);
            }
            info_done = true;
        } else if is_id3(c) && id3.is_some() {
            if !id3_done {
                out.extend_from_slice(&chunk(&id3_id, id3.unwrap()));
            }
            id3_done = true;
        } else {
            out.extend_from_slice(&data[c.0..c.1]);
        }
    }
    if !info_done {
        out.extend_from_slice(&list);
    }
    if !id3_done {
        out.extend_from_slice(&chunk(&id3_id, id3.unwrap()));
    }
    out.extend_from_slice(&data[trailer..]);
    let riff = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&riff.to_le_bytes());
    fs::write(p, out)?;
    Ok(())
}

/// INFO items as (id, value); values that are not UTF-8 are decoded as Latin-1 for display.
pub fn read_info(p: &Path) -> Result<Vec<(String, String)>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = fs::File::open(p)?;
    let len = f.metadata()?.len();
    let mut off = 12u64;
    let mut out = vec![];
    while off + 8 <= len {
        let mut c = [0u8; 8];
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut c)?;
        let sz = u32::from_le_bytes(c[4..8].try_into()?) as u64;
        if &c[0..4] == b"LIST" && sz >= 4 && sz < 16 * 1024 * 1024 {
            let mut body = vec![0u8; sz as usize];
            f.read_exact(&mut body)?;
            if &body[0..4] == b"INFO" {
                let mut q = 4usize;
                while q + 8 <= body.len() {
                    let id = String::from_utf8_lossy(&body[q..q + 4]).to_string();
                    let n = le32(&body[q + 4..]);
                    let v = &body[q + 8..(q + 8 + n).min(body.len())];
                    let v = v.split(|b| *b == 0).next().unwrap_or(&[]);
                    let s = String::from_utf8(v.to_vec()).unwrap_or_else(|_| v.iter().map(|&b| b as char).collect());
                    out.push((id, s));
                    q += 8 + n + (n & 1);
                }
            }
        }
        off += 8 + sz + (sz & 1);
    }
    Ok(out)
}
