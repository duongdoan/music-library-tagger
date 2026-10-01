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

/// `updates`: (INFO id, new value); an empty value removes the item.
pub fn write_info(p: &Path, updates: &[(&str, &str)]) -> Result<()> {
    let data = fs::read(p)?;
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        bail!("không phải WAV");
    }
    // split into chunks, keeping their raw bytes
    let mut chunks: Vec<(usize, usize)> = vec![]; // (start, end incl. padding)
    let mut off = 12;
    while off + 8 <= data.len() {
        let sz = le32(&data[off + 4..]);
        let end = (off + 8 + sz + (sz & 1)).min(data.len());
        chunks.push((off, end));
        off = end;
    }
    let is_info = |(s, _): &(usize, usize)| &data[*s..*s + 4] == b"LIST" && data.len() >= *s + 12 && &data[*s + 8..*s + 12] == b"INFO";
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
        items.retain(|(k, _)| *k != id);
        if !val.is_empty() {
            items.push((id, val.as_bytes().to_vec()));
        }
    }
    let body = info_body(&items);
    let mut list = b"LIST".to_vec();
    list.extend_from_slice(&(body.len() as u32).to_le_bytes());
    list.extend_from_slice(&body);

    let mut out = data[0..12].to_vec();
    let mut placed = false;
    for c in &chunks {
        if is_info(c) {
            if !placed && !items.is_empty() {
                out.extend_from_slice(&list);
            }
            placed = true;
        } else {
            out.extend_from_slice(&data[c.0..c.1]);
        }
    }
    if !placed && !items.is_empty() {
        out.extend_from_slice(&list);
    }
    let riff = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&riff.to_le_bytes());
    fs::write(p, out)?;
    Ok(())
}
