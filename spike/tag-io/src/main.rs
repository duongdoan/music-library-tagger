//! Spike: can Rust (lofty + a small DSF module) read and write this library safely?
//!
//!   probe <file>...                 read tags, print fields and audio hash
//!   writetest <out_dir> <file>...   copy each file, write test values atomically,
//!                                   verify, restore, verify nothing else changed
//!   list <root> <threads>           parallel directory listing speed
//!   readbench <listfile> <prefix> <n> <threads>   tag read throughput
mod dsf;
mod payload;
mod riff;

use anyhow::{anyhow, Context, Result};
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey, ItemValue, Tag, TagType};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use unicode_normalization::UnicodeNormalization;

const AUDIO: &[&str] = &["flac", "wav", "dsf", "m4a", "aiff", "aif", "ape", "mp3", "ogg", "opus", "wv"];

fn ext(p: &Path) -> String {
    p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Fields {
    title: String,
    artist: String,
    album: String,
    album_artist: String,
    genre: String,
}

fn nfc(s: &str) -> String {
    s.nfc().collect()
}

// ---------- reading ----------

fn fields_from_tag(t: &Tag) -> Fields {
    Fields {
        title: t.title().map(|s| nfc(&s)).unwrap_or_default(),
        artist: t.artist().map(|s| nfc(&s)).unwrap_or_default(),
        album: t.album().map(|s| nfc(&s)).unwrap_or_default(),
        album_artist: t.get_string(ItemKey::AlbumArtist).map(nfc).unwrap_or_default(),
        genre: t.genre().map(|s| nfc(&s)).unwrap_or_default(),
    }
}

fn fields_from_id3(t: &id3::Tag) -> Fields {
    use id3::TagLike;
    Fields {
        title: t.title().map(nfc).unwrap_or_default(),
        artist: t.artist().map(nfc).unwrap_or_default(),
        album: t.album().map(nfc).unwrap_or_default(),
        album_artist: t.album_artist().map(nfc).unwrap_or_default(),
        genre: t.genre().map(nfc).unwrap_or_default(),
    }
}


/// Scan-path FLAC reader: one 128 KB read, skip PICTURE blocks without reading them.
fn flac_fast(p: &Path) -> Result<Fields> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(p)?;
    let mut buf = vec![0u8; 128 * 1024];
    let n = f.read(&mut buf)?;
    buf.truncate(n);
    let mut off = if buf.starts_with(b"ID3") {
        10 + (((buf[6] as usize & 127) << 21) | ((buf[7] as usize & 127) << 14) | ((buf[8] as usize & 127) << 7) | (buf[9] as usize & 127))
    } else { 0 };
    let mut fields = Fields::default();
    let mut need = |off: usize, len: usize, buf: &mut Vec<u8>, f: &mut File| -> Result<Vec<u8>> {
        if off + len <= buf.len() { return Ok(buf[off..off + len].to_vec()); }
        let mut v = vec![0u8; len];
        f.seek(SeekFrom::Start(off as u64))?;
        f.read_exact(&mut v)?;
        Ok(v)
    };
    let magic = need(off, 4, &mut buf, &mut f)?;
    if magic != b"fLaC" { anyhow::bail!("not flac"); }
    off += 4;
    loop {
        let h = need(off, 4, &mut buf, &mut f)?;
        let len = ((h[1] as usize) << 16) | ((h[2] as usize) << 8) | h[3] as usize;
        if h[0] & 0x7f == 4 {
            let b = need(off + 4, len, &mut buf, &mut f)?;
            let u32le = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().unwrap()) as usize;
            let mut q = 4 + u32le(0);
            let count = u32le(q);
            q += 4;
            for _ in 0..count {
                let l = u32le(q);
                let kv = String::from_utf8_lossy(&b[q + 4..q + 4 + l]).to_string();
                q += 4 + l;
                if let Some((k, v)) = kv.split_once('=') {
                    let v = nfc(v);
                    match k.to_ascii_uppercase().as_str() {
                        "TITLE" => fields.title = v,
                        "ARTIST" => fields.artist = v,
                        "ALBUM" => fields.album = v,
                        "ALBUMARTIST" | "ALBUM ARTIST" => fields.album_artist = v,
                        "GENRE" => fields.genre = v,
                        _ => {}
                    }
                }
            }
        }
        off += 4 + len;
        if h[0] & 0x80 != 0 { break; }
    }
    Ok(fields)
}

/// Fields as the app would show them (APP-TAG-R10: ID3 wins for WAV).
fn read_fields(p: &Path) -> Result<Fields> {
    if ext(p) == "dsf" {
        return Ok(dsf::read_tag(p)?.map(|t| fields_from_id3(&t)).unwrap_or_default());
    }
    if ext(p) == "flac" && std::env::var("MLT_FAST").is_ok() {
        return flac_fast(p);
    }
    let tf = if std::env::var("MLT_BUF").is_ok() {
        // one large buffered read instead of many small SMB round trips
        let r = std::io::BufReader::with_capacity(256 * 1024, File::open(p)?);
        let props = std::env::var("MLT_NOPROPS").is_err();
        let art = std::env::var("MLT_NOART").is_err();
        lofty::probe::Probe::new(r).options(ParseOptions::new().read_properties(props).read_cover_art(art)).guess_file_type()?.read()?
    } else {
        lofty::read_from_path(p)?
    };
    let t = if ext(p) == "wav" { tf.tag(TagType::Id3v2).or_else(|| tf.tag(TagType::RiffInfo)) } else { tf.primary_tag().or_else(|| tf.first_tag()) };
    Ok(t.map(fields_from_tag).unwrap_or_default())
}

/// RIFF INFO fields read straight from the file, independent of lofty.
fn riff_info(p: &Path) -> Result<Vec<(String, String)>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(p)?;
    let len = f.metadata()?.len();
    let mut off = 12u64;
    let mut out = vec![];
    while off + 8 <= len {
        let mut c = [0u8; 8];
        f.seek(SeekFrom::Start(off))?;
        f.read_exact(&mut c)?;
        let sz = u32::from_le_bytes(c[4..8].try_into()?) as u64;
        if &c[0..4] == b"LIST" {
            let mut body = vec![0u8; sz as usize];
            f.read_exact(&mut body)?;
            if &body[0..4] == b"INFO" {
                let mut q = 4usize;
                while q + 8 <= body.len() {
                    let id = String::from_utf8_lossy(&body[q..q + 4]).to_string();
                    let n = u32::from_le_bytes(body[q + 4..q + 8].try_into()?) as usize;
                    let v = &body[q + 8..(q + 8 + n).min(body.len())];
                    let v = v.split(|b| *b == 0).next().unwrap_or(&[]);
                    out.push((id, String::from_utf8(v.to_vec()).unwrap_or_else(|_| format!("<non-utf8 {}>", String::from_utf8_lossy(v)))));
                    q += 8 + n + (n & 1);
                }
            }
        }
        off += 8 + sz + (sz & 1);
    }
    Ok(out)
}

/// Every tag item the file carries, read with an independent parser where possible.
/// Used to prove a write + restore cycle leaves nothing else changed (APP-TAG-R9).
fn inventory(p: &Path) -> Result<BTreeSet<String>> {
    let mut s = BTreeSet::new();
    let id3_frames = |t: &id3::Tag, s: &mut BTreeSet<String>, pre: &str| {
        for fr in t.frames() {
            s.insert(format!("{pre}{}={}", fr.id(), fr.content().to_string().trim_end_matches('\0')));
        }
    };
    match ext(p).as_str() {
        "wav" => {
            if let Ok(t) = id3::Tag::read_from_path(p) {
                id3_frames(&t, &mut s, "id3:");
            }
            for (k, v) in riff_info(p)? {
                s.insert(format!("info:{k}={v}"));
            }
        }
        "aif" | "aiff" => {
            if let Ok(t) = id3::Tag::read_from_path(p) {
                id3_frames(&t, &mut s, "id3:");
            }
        }
        "dsf" => {
            if let Some(t) = dsf::read_tag(p)? {
                id3_frames(&t, &mut s, "id3:");
            }
        }
        "flac" => {
            let mut f = File::open(p)?;
            let ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new())?;
            if let Some(vc) = ff.vorbis_comments() {
                for (k, v) in vc.items() {
                    s.insert(format!("vc:{}={}", k.to_ascii_uppercase(), v));
                }
            }
            for t in lofty::read_from_path(p)?.tags() {
                for pic in t.pictures() {
                    s.insert(format!("pic:{:?}:{}", pic.pic_type(), pic.data().len()));
                }
            }
        }
        "m4a" | "mp4" => {
            let mut f = File::open(p)?;
            let mf = lofty::mp4::Mp4File::read_from(&mut f, ParseOptions::new())?;
            if let Some(il) = mf.ilst() {
                for a in il {
                    s.insert(format!("mp4:{:?}={:?}", a.ident(), a.data().collect::<Vec<_>>()).chars().take(200).collect());
                }
            }
        }
        _ => {
            let tf = lofty::read_from_path(p)?;
            for t in tf.tags() {
                for it in t.items() {
                    let v = match it.value() {
                        ItemValue::Text(x) | ItemValue::Locator(x) => x.clone(),
                        ItemValue::Binary(b) => format!("<{} bytes>", b.len()),
                    };
                    s.insert(format!("{:?}:{:?}={}", t.tag_type(), it.key(), v));
                }
                s.insert(format!("{:?}:pictures={}", t.tag_type(), t.pictures().len()));
            }
        }
    }
    Ok(s)
}

// ---------- writing ----------

/// APP-WRITE-R5: write into a copy in the same directory, then rename over the original.
fn atomic_write(p: &Path, f: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    // keep the real extension last: lofty picks the format from it
    let tmp = p.with_file_name(format!(".mlt-tmp.{}", p.file_name().unwrap().to_string_lossy()));
    fs::copy(p, &tmp)?;
    if let Err(e) = f(&tmp) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    fs::rename(&tmp, p)?;
    Ok(())
}

fn set_lofty(t: &mut Tag, v: &Fields) {
    let put = |t: &mut Tag, k: ItemKey, s: &str| {
        if s.is_empty() {
            t.remove_key(k);
        } else {
            t.insert_text(k, s.to_string());
        }
    };
    put(t, ItemKey::TrackTitle, &v.title);
    put(t, ItemKey::TrackArtist, &v.artist);
    put(t, ItemKey::AlbumTitle, &v.album);
    put(t, ItemKey::AlbumArtist, &v.album_artist); // RIFF INFO has no slot: insert_text is a no-op there
    put(t, ItemKey::Genre, &v.genre);
}

/// ID3 frames via the `id3` crate, which keeps every frame it does not touch.
fn set_id3(t: &mut id3::Tag, v: &Fields) {
    use id3::TagLike;
    let put = |t: &mut id3::Tag, id: &str, s: &str| {
        if s.is_empty() {
            t.remove(id);
        } else {
            t.set_text(id, s);
        }
    };
    put(t, "TIT2", &v.title);
    put(t, "TPE1", &v.artist);
    put(t, "TALB", &v.album);
    put(t, "TPE2", &v.album_artist);
    put(t, "TCON", &v.genre);
    // foobar-style duplicate "TXXX:Album Artist": keep it in step when present
    let desc: Vec<String> = t.extended_texts().filter(|x| x.description.eq_ignore_ascii_case("album artist")).map(|x| x.description.clone()).collect();
    for d in desc {
        t.remove_extended_text(Some(&d), None);
        if !v.album_artist.is_empty() {
            t.add_frame(id3::frame::ExtendedText { description: d, value: v.album_artist.clone() });
        }
    }
}

fn id3_version(t: &id3::Tag) -> id3::Version {
    if t.version() == id3::Version::Id3v22 { id3::Version::Id3v23 } else { t.version() }
}

fn read_id3_or_new(r: id3::Result<id3::Tag>) -> Result<id3::Tag> {
    match r {
        Ok(t) => Ok(t),
        Err(e) if matches!(e.kind, id3::ErrorKind::NoTag) => Ok(id3::Tag::with_version(id3::Version::Id3v24)),
        Err(e) => Err(e.into()),
    }
}

fn write_fields(p: &Path, v: &Fields) -> Result<()> {
    use lofty::tag::TagExt;
    match ext(p).as_str() {
        "dsf" => {
            let mut t = dsf::read_tag(p)?.unwrap_or_else(|| id3::Tag::with_version(id3::Version::Id3v24));
            set_id3(&mut t, v);
            dsf::write_tag(p, &t)
        }
        "aif" | "aiff" => {
            let mut t = read_id3_or_new(id3::Tag::read_from_path(p))?;
            set_id3(&mut t, v);
            t.write_to_aiff_path(p, id3_version(&t))?;
            Ok(())
        }
        "wav" => {
            // APP-TAG-R10: RIFF INFO first (own writer, UTF-8, keeps untouched raw bytes),
            // then the ID3 chunk (id3 crate).
            riff::write_info(p, &[("INAM", &v.title), ("IART", &v.artist), ("IPRD", &v.album), ("IGNR", &v.genre)])?;
            let mut t = read_id3_or_new(id3::Tag::read_from_path(p))?;
            set_id3(&mut t, v);
            t.write_to_wav_path(p, id3_version(&t))?;
            Ok(())
        }
        "flac" => {
            let mut f = File::open(p)?;
            let mut ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new())?;
            drop(f);
            if ff.vorbis_comments().is_none() {
                ff.set_vorbis_comments(lofty::ogg::tag::VorbisComments::default());
            }
            let vc = ff.vorbis_comments_mut().unwrap();
            // update every spelling a file already uses, e.g. both ALBUMARTIST and "ALBUM ARTIST"
            for (keys, val) in [
                (&["TITLE"][..], &v.title),
                (&["ARTIST"][..], &v.artist),
                (&["ALBUM"][..], &v.album),
                (&["ALBUMARTIST", "ALBUM ARTIST", "ALBUM_ARTIST"][..], &v.album_artist),
                (&["GENRE"][..], &v.genre),
            ] {
                let present: Vec<String> = vc.items().map(|(k, _)| k.to_string()).filter(|k| keys.iter().any(|x| x.eq_ignore_ascii_case(k))).collect();
                let targets = if present.is_empty() { vec![keys[0].to_string()] } else { present };
                for k in targets {
                    let _ = vc.remove(&k).count();
                    if !val.is_empty() {
                        vc.push(k, val.clone());
                    }
                }
            }
            ff.save_to_path(p, WriteOptions::default())?;
            Ok(())
        }
        "m4a" | "mp4" => {
            use lofty::mp4::{Atom, AtomData, AtomIdent};
            let mut f = File::open(p)?;
            let mut mf = lofty::mp4::Mp4File::read_from(&mut f, ParseOptions::new())?;
            drop(f);
            if mf.ilst().is_none() {
                mf.set_ilst(lofty::mp4::Ilst::default());
            }
            let il = mf.ilst_mut().unwrap();
            for (code, val) in [(*b"\xa9nam", &v.title), (*b"\xa9ART", &v.artist), (*b"\xa9alb", &v.album), (*b"aART", &v.album_artist), (*b"\xa9gen", &v.genre)] {
                let id = AtomIdent::Fourcc(code);
                let _ = il.remove(&id).count();
                if !val.is_empty() {
                    il.insert(Atom::new(id, AtomData::UTF8(val.clone())));
                }
            }
            mf.save_to_path(p, WriteOptions::default())?;
            Ok(())
        }
        _ => {
            let mut tf = lofty::read_from_path(p)?;
            let ty = tf.primary_tag_type();
            if tf.tag(ty).is_none() {
                tf.insert_tag(Tag::new(ty));
            }
            set_lofty(tf.tag_mut(ty).unwrap(), v);
            tf.save_to_path(p, WriteOptions::default())?;
            Ok(())
        }
    }
}

fn writetest(out: &Path, files: &[PathBuf]) -> Result<()> {
    fs::create_dir_all(out)?;
    println!("| file | write | fields ok | audio unchanged | restore: tags identical | RIFF INFO after write |");
    println!("|---|---|---|---|---|---|");
    for src in files {
        let name = src.file_name().unwrap().to_string_lossy().to_string();
        let p = out.join(&name);
        fs::copy(src, &p).with_context(|| format!("copy {name}"))?;
        let short: String = name.chars().take(48).collect();
        let r: Result<String> = (|| {
            let orig = read_fields(&p)?;
            let inv0 = inventory(&p)?;
            let h0 = payload::hash(&p)?;
            let test = Fields {
                title: format!("{} [spike]", orig.title),
                artist: "Nghệ Sĩ Thử Nghiệm".into(),
                album: orig.album.clone(),
                album_artist: "Various Artists (spike)".into(),
                genre: "Nhạc Trịnh".into(),
            };
            let t0 = Instant::now();
            atomic_write(&p, |tmp| write_fields(tmp, &test))?;
            let ms = t0.elapsed().as_millis();
            let got = read_fields(&p)?;
            let ok = got == test;
            let h1 = payload::hash(&p)?;
            let info = if ext(&p) == "wav" {
                let i = riff_info(&p)?;
                let g = |k: &str| i.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()).unwrap_or_default();
                format!("INAM={} IART={}", g("INAM"), g("IART"))
            } else {
                "-".into()
            };
            // restore the original values and compare every tag item
            atomic_write(&p, |tmp| write_fields(tmp, &orig))?;
            let inv1 = inventory(&p)?;
            let h2 = payload::hash(&p)?;
            let lost: Vec<_> = inv0.difference(&inv1).cloned().collect();
            let added: Vec<_> = inv1.difference(&inv0).cloned().collect();
            let same = if lost.is_empty() && added.is_empty() {
                format!("yes ({} items)", inv0.len())
            } else {
                format!("NO: lost {:?} added {:?}", lost.iter().take(4).collect::<Vec<_>>(), added.iter().take(4).collect::<Vec<_>>())
            };
            Ok(format!(
                "{ms} ms | {} | {} | {} | {}",
                if ok { "yes".to_string() } else { format!("NO {got:?}") },
                if h0 == h1 && h1 == h2 { "yes" } else { "NO" },
                same,
                info
            ))
        })();
        match r {
            Ok(s) => println!("| {short} | {s} |"),
            Err(e) => println!("| {short} | ERROR: {e:#} | | | | |"),
        }
    }
    Ok(())
}

// ---------- speed ----------

fn walk(dir: &Path, n: &AtomicUsize) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut subdirs = vec![];
    for e in rd.flatten() {
        let name = e.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "roon-backup" {
            continue;
        }
        match e.file_type() {
            Ok(t) if t.is_dir() => subdirs.push(e.path()),
            Ok(_) => {
                if std::env::var("MLT_STAT").is_ok() {
                    // incremental scan needs mtime + size of every file
                    if let Ok(m) = e.metadata() {
                        std::hint::black_box((m.len(), m.modified().ok()));
                    }
                }
                n.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {}
        }
    }
    subdirs.par_iter().for_each(|d| walk(d, n));
}

fn list(root: &Path, threads: usize) -> Result<()> {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let n = AtomicUsize::new(0);
    let t = Instant::now();
    pool.install(|| walk(root, &n));
    let s = t.elapsed().as_secs_f64();
    let n = n.load(Ordering::Relaxed);
    println!("list threads={threads}: {n} files in {s:.1}s = {:.0} files/s", n as f64 / s);
    Ok(())
}

fn readbench(listfile: &Path, prefix: &Path, n: usize, threads: usize, offset: usize) -> Result<()> {
    let all: Vec<PathBuf> = fs::read_to_string(listfile)?
        .lines()
        .map(|l| prefix.join(l))
        .filter(|p| AUDIO.contains(&ext(p).as_str()))
        .filter(|p| std::env::var("MLT_EXT").map(|e| ext(p) == e).unwrap_or(true))
        .collect();
    let step = (all.len() / n).max(1);
    let pick: Vec<_> = all.iter().skip(offset).step_by(step).take(n).cloned().collect();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let errs = AtomicUsize::new(0);
    let t = Instant::now();
    pool.install(|| {
        pick.par_iter().for_each(|p| {
            if read_fields(p).is_err() {
                errs.fetch_add(1, Ordering::Relaxed);
            }
        })
    });
    let s = t.elapsed().as_secs_f64();
    println!(
        "readbench threads={threads}: {} files in {s:.1}s = {:.0} files/s, errors {}",
        pick.len(),
        pick.len() as f64 / s,
        errs.load(Ordering::Relaxed)
    );
    Ok(())
}


fn fmtbench(listfile: &Path, prefix: &Path, per: usize, offset: usize) -> Result<()> {
    use std::io::Read;
    let lines = fs::read_to_string(listfile)?;
    for e in ["flac", "wav", "dsf", "m4a", "aiff"] {
        let all: Vec<PathBuf> = lines.lines().map(|l| prefix.join(l)).filter(|p| ext(p) == e).collect();
        let step = (all.len() / per).max(1);
        let pick: Vec<_> = all.iter().skip(offset).step_by(step).take(per).cloned().collect();
        let mut t = [0f64; 3];
        let mut errs = 0;
        let mut sizes = 0u64;
        for p in &pick {
            sizes += fs::metadata(p).map(|m| m.len()).unwrap_or(0);
            let a = Instant::now();
            let mut buf = vec![0u8; 65536];
            if let Ok(mut f) = File::open(p) { let _ = f.read(&mut buf); }
            t[0] += a.elapsed().as_secs_f64();
            let a = Instant::now();
            if lofty::probe::Probe::open(p).and_then(|x| x.options(ParseOptions::new().read_properties(false)).read()).is_err() { errs += 1; }
            t[1] += a.elapsed().as_secs_f64();
            let a = Instant::now();
            let _ = lofty::read_from_path(p);
            t[2] += a.elapsed().as_secs_f64();
        }
        let n = pick.len() as f64;
        println!("{e:5} n={} avg size {:.0} MB | first 64KB {:.0} ms | lofty no-props {:.0} ms | lofty full {:.0} ms | errors {errs}",
            pick.len(), sizes as f64 / n / 1e6, t[0] / n * 1e3, t[1] / n * 1e3, t[2] / n * 1e3);
    }
    Ok(())
}

fn probe(files: &[PathBuf]) -> Result<()> {
    for p in files {
        let r: Result<String> = (|| {
            let f = read_fields(p)?;
            let h = payload::hash(p).unwrap_or_else(|e| format!("hash? {e}"));
            Ok(format!("{f:?} audio={h}"))
        })();
        println!("{} => {}", p.file_name().unwrap().to_string_lossy(), r.unwrap_or_else(|e| format!("ERROR {e:#}")));
    }
    Ok(())
}

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let paths = |from: usize| a[from..].iter().map(PathBuf::from).collect::<Vec<_>>();
    match a.get(1).map(String::as_str) {
        Some("probe") => probe(&paths(2)),
        Some("fmtbench") => fmtbench(Path::new(&a[2]), Path::new(&a[3]), a[4].parse()?, a[5].parse()?),
        Some("writetest") => writetest(Path::new(&a[2]), &paths(3)),
        Some("list") => list(Path::new(&a[2]), a[3].parse()?),
        Some("readbench") => readbench(Path::new(&a[2]), Path::new(&a[3]), a[4].parse()?, a[5].parse()?, a.get(6).map(|x| x.parse()).transpose()?.unwrap_or(0)),
        _ => Err(anyhow!("usage: probe|writetest|list|readbench …")),
    }
}
