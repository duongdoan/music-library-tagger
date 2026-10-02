//! Cover images for the inspector and the table's thumbnail column.
//!
//! Source order: the front cover embedded in the file, else an image file in the
//! album folder (cover/folder/front .jpg/.jpeg/.png). Thumbnails are cached on the
//! local disk keyed by file path + modification time, so an image on the NAS is
//! read over the network once.
use crate::tags;
use anyhow::{bail, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cover {
    /// data: URL of a JPEG thumbnail no larger than the requested size
    pub url: String,
    /// original image size in pixels and bytes
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    /// "embedded" | "folder"
    pub source: String,
    /// folder image file name when source == "folder"
    pub file: Option<String>,
}

const FOLDER_NAMES: &[&str] = &["cover", "folder", "front"];
const FOLDER_EXT: &[&str] = &["jpg", "jpeg", "png"];

/// Folder image per directory, remembered for the session (one listing per folder).
pub struct FolderImages(Mutex<HashMap<PathBuf, Option<PathBuf>>>);

impl FolderImages {
    pub fn new() -> Self {
        FolderImages(Mutex::new(HashMap::new()))
    }
    fn find(&self, dir: &Path) -> Option<PathBuf> {
        if let Some(v) = self.0.lock().unwrap().get(dir) {
            return v.clone();
        }
        let mut best: Option<(usize, PathBuf)> = None;
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.starts_with('.') {
                    continue;
                }
                let (stem, ext) = match name.rsplit_once('.') {
                    Some(x) => x,
                    None => continue,
                };
                if !FOLDER_EXT.contains(&ext) {
                    continue;
                }
                if let Some(rank) = FOLDER_NAMES.iter().position(|n| *n == stem) {
                    if best.as_ref().map(|(r, _)| rank < *r).unwrap_or(true) {
                        best = Some((rank, p));
                    }
                }
            }
        }
        let v = best.map(|(_, p)| p);
        self.0.lock().unwrap().insert(dir.to_path_buf(), v.clone());
        v
    }
}

/// Bytes of the embedded front cover (or the first picture), without reading the audio.
pub fn embedded(p: &Path) -> Result<Option<Vec<u8>>> {
    match tags::ext(p).as_str() {
        "flac" => flac_picture(p),
        "dsf" => Ok(tags::dsf::read_tag(p)?.and_then(|t| pick_id3(&t))),
        "wav" | "aif" | "aiff" | "mp3" => Ok(id3::Tag::read_from_path(p).ok().and_then(|t| pick_id3(&t))),
        "wma" => Ok(None),
        _ => {
            use lofty::file::TaggedFileExt;
            let tf = lofty::read_from_path(p)?;
            let pics: Vec<_> = tf.tags().iter().flat_map(|t| t.pictures().iter()).collect();
            let front = pics.iter().find(|x| x.pic_type() == lofty::picture::PictureType::CoverFront).or(pics.first());
            Ok(front.map(|x| x.data().to_vec()))
        }
    }
}

fn pick_id3(t: &id3::Tag) -> Option<Vec<u8>> {
    let pics: Vec<_> = t.pictures().collect();
    pics.iter()
        .find(|x| x.picture_type == id3::frame::PictureType::CoverFront)
        .or(pics.first())
        .map(|x| x.data.clone())
}

/// Walk FLAC metadata block headers and read only the chosen PICTURE block.
fn flac_picture(p: &Path) -> Result<Option<Vec<u8>>> {
    let mut f = File::open(p)?;
    let mut head = [0u8; 10];
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
        bail!("không phải FLAC");
    }
    off += 4;
    let mut chosen: Option<(u64, usize)> = None;
    for _ in 0..1000 {
        let mut h = [0u8; 4];
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut h)?;
        let len = ((h[1] as usize) << 16) | ((h[2] as usize) << 8) | h[3] as usize;
        if h[0] & 0x7f == 6 {
            let mut t = [0u8; 4];
            f.read_exact(&mut t)?;
            let front = u32::from_be_bytes(t) == 3;
            if chosen.is_none() || front {
                chosen = Some((off + 4, len));
            }
            if front {
                break;
            }
        }
        off += 4 + len as u64;
        if h[0] & 0x80 != 0 {
            break;
        }
    }
    let Some((at, len)) = chosen else { return Ok(None) };
    let mut b = vec![0u8; len];
    f.seek(SeekFrom::Start(at))?;
    f.read_exact(&mut b)?;
    // PICTURE: type, mime, description, width, height, depth, colours, data
    let u = |i: usize| -> Result<usize> { Ok(u32::from_be_bytes(b.get(i..i + 4).ok_or_else(|| anyhow::anyhow!("PICTURE lỗi"))?.try_into()?) as usize) };
    let mut q = 4;
    q += 4 + u(q)?;
    q += 4 + u(q)?;
    q += 16;
    let n = u(q)?;
    Ok(b.get(q + 4..q + 4 + n).map(|d| d.to_vec()))
}

fn thumbnail(bytes: &[u8], size: u32) -> Result<(String, u32, u32)> {
    let img = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;
    let (w, h) = (img.width(), img.height());
    let t = if w > size || h > size { img.thumbnail(size, size) } else { img };
    let mut out = Vec::new();
    t.to_rgb8().write_to(&mut Cursor::new(&mut out), image::ImageFormat::Jpeg)?;
    let url = format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(&out));
    Ok((url, w, h))
}

fn cache_key(p: &Path, size: u32) -> String {
    let mtime = fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
    let mut h = Sha256::new();
    h.update(p.to_string_lossy().as_bytes());
    h.update(mtime.to_le_bytes());
    h.update(size.to_le_bytes());
    format!("{:x}", h.finalize())[..32].to_string()
}

fn folder_cover(img: &Path, size: u32, cache_dir: &Path) -> Result<Option<Cover>> {
    let cached = cache_dir.join(format!("{}.json", cache_key(img, size)));
    if let Ok(s) = fs::read_to_string(&cached) {
        if let Ok(c) = serde_json::from_str::<Option<Cover>>(&s) {
            return Ok(c);
        }
    }
    let bytes = fs::read(img)?;
    let (url, width, height) = thumbnail(&bytes, size)?;
    let file = img.file_name().map(|n| n.to_string_lossy().to_string());
    let c = Some(Cover { url, width, height, bytes: bytes.len() as u64, source: "folder".into(), file });
    fs::create_dir_all(cache_dir).ok();
    let _ = fs::write(&cached, serde_json::to_string(&c)?);
    Ok(c)
}

/// Cover for one track at `size` px (longest side). None when there is no image.
pub fn get(p: &Path, size: u32, cache_dir: &Path, folders: &FolderImages) -> Result<Option<Cover>> {
    let key = cache_key(p, size);
    let cached = cache_dir.join(format!("{key}.json"));
    if let Ok(s) = fs::read_to_string(&cached) {
        if let Ok(c) = serde_json::from_str::<Option<Cover>>(&s) {
            return Ok(c);
        }
    }
    let cover = match embedded(p).ok().flatten() {
        Some(bytes) => {
            let (url, width, height) = thumbnail(&bytes, size)?;
            Some(Cover { url, width, height, bytes: bytes.len() as u64, source: "embedded".into(), file: None })
        }
        None => match p.parent().and_then(|d| folders.find(d)) {
            // one thumbnail per folder image, shared by every track of the album
            Some(img) => folder_cover(&img, size, cache_dir)?,
            None => None,
        },
    };
    fs::create_dir_all(cache_dir).ok();
    let _ = fs::write(&cached, serde_json::to_string(&cover)?);
    Ok(cover)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(w, h, image::Rgb([10, 120, 120]));
        let mut out = Vec::new();
        img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png).unwrap();
        out
    }

    /// FLAC with STREAMINFO + one PICTURE block (front cover).
    fn flac_with_picture(dir: &Path, data: &[u8]) -> PathBuf {
        let mut pic = vec![];
        pic.extend(3u32.to_be_bytes());
        pic.extend(9u32.to_be_bytes());
        pic.extend(b"image/png");
        pic.extend(0u32.to_be_bytes());
        for v in [0u32, 0, 24, 0] {
            pic.extend(v.to_be_bytes());
        }
        pic.extend((data.len() as u32).to_be_bytes());
        pic.extend(data);
        let mut f = b"fLaC".to_vec();
        f.push(0);
        f.extend(&34u32.to_be_bytes()[1..]);
        f.extend([0u8; 34]);
        f.push(0x80 | 6);
        f.extend(&(pic.len() as u32).to_be_bytes()[1..]);
        f.extend(&pic);
        let p = dir.join("a.flac");
        fs::write(&p, f).unwrap();
        p
    }

    #[test]
    fn embedded_cover_is_thumbnailed_and_cached() {
        let d = tempfile::tempdir().unwrap();
        let p = flac_with_picture(d.path(), &png(1200, 1000));
        let cache = d.path().join("thumbs");
        let fi = FolderImages::new();
        let c = get(&p, 64, &cache, &fi).unwrap().unwrap();
        assert_eq!((c.width, c.height, c.source.as_str()), (1200, 1000, "embedded"));
        assert!(c.url.starts_with("data:image/jpeg;base64,"));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 1, "cached on disk");
        // served from cache even if the file becomes unreadable
        let c2 = get(&p, 64, &cache, &fi).unwrap().unwrap();
        assert_eq!(c2.url, c.url);
    }

    #[test]
    fn folder_image_is_used_when_nothing_is_embedded() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("01.wav");
        fs::write(&p, b"RIFF\x04\0\0\0WAVE").unwrap();
        fs::write(d.path().join("Folder.JPG.txt"), b"not an image").unwrap();
        fs::write(d.path().join("Cover.png"), png(500, 500)).unwrap();
        let c = get(&p, 300, &d.path().join("t"), &FolderImages::new()).unwrap().unwrap();
        assert_eq!((c.source.as_str(), c.file.as_deref(), c.width), ("folder", Some("Cover.png"), 500));
    }

    /// `MLT_SAMPLES=dir cargo test real_covers -- --nocapture`
    #[test]
    fn real_covers() {
        let Ok(dir) = std::env::var("MLT_SAMPLES") else { return };
        let cache = tempfile::tempdir().unwrap();
        let fi = FolderImages::new();
        for e in fs::read_dir(dir).unwrap().flatten() {
            let t = std::time::Instant::now();
            let r = get(&e.path(), 600, cache.path(), &fi);
            let d = t.elapsed().as_millis();
            match r {
                Ok(Some(c)) => println!("{:<50} {}x{} {} KB {} ({} ms, thumb {} KB)", e.file_name().to_string_lossy().chars().take(50).collect::<String>(), c.width, c.height, c.bytes / 1024, c.source, d, c.url.len() / 1024),
                Ok(None) => println!("{:<50} (no cover) ({d} ms)", e.file_name().to_string_lossy().chars().take(50).collect::<String>()),
                Err(x) => println!("{:<50} ERROR {x}", e.file_name().to_string_lossy().chars().take(50).collect::<String>()),
            }
        }
    }

    #[test]
    fn no_image_is_none() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("01.wav");
        fs::write(&p, b"RIFF\x04\0\0\0WAVE").unwrap();
        assert!(get(&p, 64, &d.path().join("t"), &FolderImages::new()).unwrap().is_none());
    }
}
