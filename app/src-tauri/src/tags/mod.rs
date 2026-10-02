//! Reading and writing tags, one code path per container (spike-report §1).
//!
//! Writes never go through lofty's generic `TaggedFile` for formats where that
//! would drop unknown items: FLAC and MP4 use lofty's concrete types, ID3 based
//! formats (WAV, AIFF, DSF, MP3) use the `id3` crate, RIFF INFO and DSF use our
//! own small modules.
pub mod dsf;
pub mod flac_inplace;
#[cfg(test)]
pub mod payload;
pub mod riff;

use crate::model::{Fields, TrackTags, FIELDS};
use crate::norm::{canonical, normalize};
use anyhow::{bail, Result};
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey, Tag, TagType};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn ext(p: &Path) -> String {
    p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase()
}

fn put(f: &mut Fields, k: &str, v: impl AsRef<str>) {
    let v = normalize(v.as_ref());
    if !v.is_empty() {
        f.insert(k.to_string(), v);
    }
}

/// "3/12" -> ("3", Some("12"))
fn split_slash(v: &str) -> (String, Option<String>) {
    match v.split_once('/') {
        Some((a, b)) => (a.trim().to_string(), Some(b.trim().to_string())),
        None => (v.trim().to_string(), None),
    }
}

fn num(v: &str) -> String {
    canonical("track", v)
}

// ---------------------------------------------------------------- reading

pub fn read_track(p: &Path) -> Result<TrackTags> {
    match ext(p).as_str() {
        "flac" => read_flac_fast(p),
        "dsf" => read_dsf(p),
        "wav" => read_wav(p),
        _ => read_generic(p),
    }
}

fn fields_from_lofty(t: &Tag) -> Fields {
    let mut f = Fields::new();
    if let Some(v) = t.title() { put(&mut f, "title", v) }
    if let Some(v) = t.artist() { put(&mut f, "artist", v) }
    if let Some(v) = t.album() { put(&mut f, "album", v) }
    if let Some(v) = t.genre() { put(&mut f, "genre", v) }
    if let Some(v) = t.comment() { put(&mut f, "comment", v) }
    if let Some(v) = t.get_string(ItemKey::AlbumArtist) { put(&mut f, "albumartist", v) }
    if let Some(v) = t.get_string(ItemKey::Composer) { put(&mut f, "composer", v) }
    if let Some(v) = t.get_string(ItemKey::Year).or_else(|| t.get_string(ItemKey::RecordingDate)) {
        put(&mut f, "year", canonical("year", v))
    }
    if let Some(v) = t.track() { put(&mut f, "track", v.to_string()) }
    if let Some(v) = t.track_total() { put(&mut f, "tracktotal", v.to_string()) }
    if let Some(v) = t.disk() { put(&mut f, "disc", v.to_string()) }
    if let Some(v) = t.disk_total() { put(&mut f, "disctotal", v.to_string()) }
    f
}

fn fields_from_id3(t: &id3::Tag) -> Fields {
    use id3::TagLike;
    let mut f = Fields::new();
    let text = |id: &str| t.get(id).and_then(|fr| fr.content().text()).map(|s| s.trim_end_matches('\0').to_string());
    if let Some(v) = t.title() { put(&mut f, "title", v) }
    if let Some(v) = t.artist() { put(&mut f, "artist", v) }
    if let Some(v) = t.album() { put(&mut f, "album", v) }
    if let Some(v) = t.album_artist() { put(&mut f, "albumartist", v) }
    if let Some(v) = text("TCOM") { put(&mut f, "composer", v) }
    if let Some(v) = t.genre_parsed() { put(&mut f, "genre", v) }
    if let Some(c) = t.comments().find(|c| c.description.is_empty()).or_else(|| t.comments().next()) {
        put(&mut f, "comment", &c.text)
    }
    if let Some(v) = text("TDRC").or_else(|| text("TYER")) { put(&mut f, "year", canonical("year", &v)) }
    if let Some(v) = t.track() { put(&mut f, "track", v.to_string()) }
    if let Some(v) = t.total_tracks() { put(&mut f, "tracktotal", v.to_string()) }
    if let Some(v) = t.disc() { put(&mut f, "disc", v.to_string()) }
    if let Some(v) = t.total_discs() { put(&mut f, "disctotal", v.to_string()) }
    f
}

/// TPE2 and a foobar-style "TXXX:Album Artist" that disagree (APP-TAG-R13).
fn id3_aa_mismatch(t: &id3::Tag) -> bool {
    use id3::TagLike;
    let tpe2 = normalize(t.album_artist().unwrap_or(""));
    t.extended_texts().filter(|x| x.description.eq_ignore_ascii_case("album artist")).any(|x| normalize(&x.value) != tpe2)
}

fn format_name(kind: &str, rate: Option<u32>, bits: Option<u8>) -> String {
    match (rate, bits) {
        (Some(r), Some(b)) if r > 0 => format!("{kind} {b}/{}", trim_rate(r)),
        (Some(r), None) if r > 0 => format!("{kind} {}", trim_rate(r)),
        _ => kind.to_string(),
    }
}

fn trim_rate(r: u32) -> String {
    let k = r as f64 / 1000.0;
    if (k - k.round()).abs() < 1e-9 { format!("{}", k as u32) } else { format!("{k:.1}") }
}

fn read_generic(p: &Path) -> Result<TrackTags> {
    let tf = lofty::read_from_path(p)?;
    let t = tf.primary_tag().or_else(|| tf.first_tag());
    let props = tf.properties();
    Ok(TrackTags {
        fields: t.map(fields_from_lofty).unwrap_or_default(),
        has_art: tf.tags().iter().any(|t| t.picture_count() > 0),
        format: format_name(&ext(p).to_uppercase(), props.sample_rate(), props.bit_depth()),
        duration_ms: Some(props.duration().as_millis() as u64),
        riff_mismatch: false,
        aa_mismatch: matches!(ext(p).as_str(), "mp3" | "aif" | "aiff")
            && id3::Tag::read_from_path(p).map(|t| id3_aa_mismatch(&t)).unwrap_or(false),
    })
}

/// RIFF INFO id for each field we keep in sync (APP-TAG-R10).
pub const INFO_MAP: &[(&str, &str)] = &[
    ("title", "INAM"), ("artist", "IART"), ("album", "IPRD"), ("genre", "IGNR"), ("year", "ICRD"), ("track", "ITRK"), ("comment", "ICMT"),
];

fn read_wav(p: &Path) -> Result<TrackTags> {
    let tf = lofty::read_from_path(p)?;
    let props = tf.properties();
    let info = riff::read_info(p).unwrap_or_default();
    let id3 = id3::Tag::read_from_path(p).ok();
    // APP-TAG-R10: ID3 is the source of truth; INFO only when there is no ID3 at all
    let fields = match &id3 {
        Some(t) => fields_from_id3(t),
        None => {
            let mut f = Fields::new();
            for (field, id) in INFO_MAP {
                let alt = if *id == "ITRK" { "IPRT" } else { id };
                if let Some((_, v)) = info.iter().find(|(k, _)| k == id || k == alt) {
                    put(&mut f, field, if *field == "track" { num(&split_slash(v).0) } else { v.clone() });
                }
            }
            f
        }
    };
    let riff_mismatch = id3.is_some()
        && !info.is_empty()
        && INFO_MAP.iter().any(|(field, id)| {
            let alt = if *id == "ITRK" { "IPRT" } else { id };
            let iv = info.iter().find(|(k, _)| k == id || k == alt).map(|(_, v)| v.as_str()).unwrap_or("");
            let iv = if *field == "track" { num(&split_slash(iv).0) } else { canonical(field, iv) };
            iv != canonical(field, fields.get(*field).map(String::as_str).unwrap_or(""))
        });
    Ok(TrackTags {
        has_art: id3.as_ref().map(|t| t.pictures().next().is_some()).unwrap_or(false),
        fields,
        format: format_name("WAV", props.sample_rate(), props.bit_depth()),
        duration_ms: Some(props.duration().as_millis() as u64),
        riff_mismatch,
        aa_mismatch: id3.as_ref().map(id3_aa_mismatch).unwrap_or(false),
    })
}

fn read_dsf(p: &Path) -> Result<TrackTags> {
    let tag = dsf::read_tag(p)?;
    let (rate, samples) = dsf::stream_info(p)?;
    let dsd = if rate > 0 { format!("DSF DSD{}", rate / 44_100) } else { "DSF".into() };
    Ok(TrackTags {
        fields: tag.as_ref().map(fields_from_id3).unwrap_or_default(),
        has_art: tag.as_ref().map(|t| t.pictures().next().is_some()).unwrap_or(false),
        aa_mismatch: tag.as_ref().map(id3_aa_mismatch).unwrap_or(false),
        format: dsd,
        duration_ms: if rate > 0 { Some(samples * 1000 / rate as u64) } else { None },
        riff_mismatch: false,
    })
}

/// Scan path for FLAC (72% of the library): one 128 KB read, PICTURE blocks
/// are skipped without reading them (spike-report §2).
fn read_flac_fast(p: &Path) -> Result<TrackTags> {
    let mut f = File::open(p)?;
    let mut head = vec![0u8; 128 * 1024];
    let n = f.read(&mut head)?;
    head.truncate(n);
    let mut get = |off: usize, len: usize| -> Result<Vec<u8>> {
        if off + len <= head.len() {
            return Ok(head[off..off + len].to_vec());
        }
        let mut v = vec![0u8; len];
        f.seek(SeekFrom::Start(off as u64))?;
        f.read_exact(&mut v)?;
        Ok(v)
    };
    let id3 = get(0, 10)?;
    let mut off = if &id3[0..3] == b"ID3" {
        10 + (((id3[6] as usize & 127) << 21) | ((id3[7] as usize & 127) << 14) | ((id3[8] as usize & 127) << 7) | (id3[9] as usize & 127))
    } else {
        0
    };
    if get(off, 4)? != b"fLaC" {
        bail!("không phải file FLAC hợp lệ");
    }
    off += 4;
    let mut t = TrackTags { format: "FLAC".into(), ..Default::default() };
    let mut multi: Vec<(String, String)> = vec![];
    loop {
        let h = get(off, 4)?;
        let len = ((h[1] as usize) << 16) | ((h[2] as usize) << 8) | h[3] as usize;
        match h[0] & 0x7f {
            0 if len >= 18 => {
                let b = get(off + 4, 18)?;
                let rate = ((b[10] as u32) << 12) | ((b[11] as u32) << 4) | ((b[12] as u32) >> 4);
                let bps = ((((b[12] & 1) << 4) | (b[13] >> 4)) + 1) as u8;
                let total = (((b[13] & 0x0f) as u64) << 32) | u32::from_be_bytes(b[14..18].try_into()?) as u64;
                t.format = format_name("FLAC", Some(rate), Some(bps));
                if rate > 0 {
                    t.duration_ms = Some(total * 1000 / rate as u64);
                }
            }
            4 => {
                let b = get(off + 4, len)?;
                let u = |i: usize| -> Result<usize> {
                    Ok(u32::from_le_bytes(b.get(i..i + 4).ok_or_else(|| anyhow::anyhow!("vorbis comment lỗi"))?.try_into()?) as usize)
                };
                let mut q = 4 + u(0)?;
                let count = u(q)?;
                q += 4;
                for _ in 0..count {
                    let l = u(q)?;
                    let kv = String::from_utf8_lossy(b.get(q + 4..q + 4 + l).unwrap_or(&[])).to_string();
                    q += 4 + l;
                    if let Some((k, v)) = kv.split_once('=') {
                        multi.push((k.to_ascii_uppercase(), v.to_string()));
                    }
                }
            }
            6 => t.has_art = true,
            _ => {}
        }
        off += 4 + len;
        if h[0] & 0x80 != 0 {
            break;
        }
    }
    // first spelling that has a value wins; repeated values of that one key are joined (APP-TAG-R7)
    let first = |keys: &[&str]| -> Option<String> {
        keys.iter().find_map(|key| {
            let vals: Vec<&str> = multi.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).filter(|v| !v.trim().is_empty()).collect();
            if vals.is_empty() { None } else { Some(vals.join("; ")) }
        })
    };
    // APP-TAG-R13: the Album Artist spellings present in the file must agree (an empty
    // "albumartist=" next to a filled "ALBUM ARTIST=" makes players disagree)
    {
        let mut seen: Vec<String> = vec![];
        for key in ["ALBUMARTIST", "ALBUM ARTIST", "ALBUM_ARTIST"] {
            let vals: Vec<String> = multi.iter().filter(|(k, _)| k == key).map(|(_, v)| normalize(v)).collect();
            if !vals.is_empty() {
                seen.push(vals.join("; "));
            }
        }
        seen.dedup();
        // an Album Artist only under "ALBUM ARTIST" / "ALBUM_ARTIST" lacks the standard key players read
        let standard = multi.iter().any(|(k, v)| k == "ALBUMARTIST" && !v.trim().is_empty());
        t.aa_mismatch = seen.len() > 1 || (!standard && seen.iter().any(|v| !v.is_empty()));
    }
    let f = &mut t.fields;
    for (field, keys) in VORBIS_KEYS {
        if let Some(v) = first(keys) {
            let v = match *field {
                "track" | "disc" => num(&split_slash(&v).0),
                "year" => canonical("year", &v),
                _ => v,
            };
            put(f, field, v);
        }
    }
    // "3/12" style TRACKNUMBER carries the total when TRACKTOTAL is absent
    for (field, total, key) in [("track", "tracktotal", "TRACKNUMBER"), ("disc", "disctotal", "DISCNUMBER")] {
        let _ = field;
        if !f.contains_key(total) {
            if let Some(Some(tt)) = first(&[key]).map(|v| split_slash(&v).1) {
                put(f, total, num(&tt));
            }
        }
    }
    Ok(t)
}

/// Vorbis comment keys per field; the first key is the one we create.
pub(crate) const VORBIS_KEYS: &[(&str, &[&str])] = &[
    ("title", &["TITLE"]),
    ("artist", &["ARTIST"]),
    ("album", &["ALBUM"]),
    ("albumartist", &["ALBUMARTIST", "ALBUM ARTIST", "ALBUM_ARTIST"]),
    ("composer", &["COMPOSER"]),
    ("genre", &["GENRE"]),
    ("year", &["DATE", "YEAR"]),
    ("track", &["TRACKNUMBER"]),
    ("tracktotal", &["TRACKTOTAL", "TOTALTRACKS"]),
    ("disc", &["DISCNUMBER"]),
    ("disctotal", &["DISCTOTAL", "TOTALDISCS"]),
    ("comment", &["COMMENT", "DESCRIPTION"]),
];

// ---------------------------------------------------------------- writing

/// Write `changes` (field -> new value, "" removes the field) into the file at `p`.
/// For WAV the RIFF INFO area is re-synced from the final ID3 values (APP-TAG-R10),
/// so an empty `changes` map is the "Đồng bộ RIFF INFO" action.
pub fn write_track(p: &Path, changes: &Fields) -> Result<()> {
    for k in changes.keys() {
        if !FIELDS.contains(&k.as_str()) {
            bail!("trường không hỗ trợ: {k}");
        }
    }
    match ext(p).as_str() {
        "dsf" => {
            let mut t = dsf::read_tag(p)?.unwrap_or_else(|| id3::Tag::with_version(id3::Version::Id3v24));
            set_id3(&mut t, changes);
            dsf::write_tag(p, &t)
        }
        "aif" | "aiff" => {
            let mut t = read_id3_or_new(id3::Tag::read_from_path(p))?;
            set_id3(&mut t, changes);
            t.write_to_aiff_path(p, id3_version(&t))?;
            Ok(())
        }
        "mp3" => {
            let mut t = read_id3_or_new(id3::Tag::read_from_path(p))?;
            set_id3(&mut t, changes);
            t.write_to_path(p, id3_version(&t))?;
            Ok(())
        }
        "wav" => {
            let had_id3 = id3::Tag::read_from_path(p).is_ok();
            let mut t = read_id3_or_new(id3::Tag::read_from_path(p))?;
            if !had_id3 {
                // first ID3 for this file: seed it from RIFF INFO (APP-TAG-R10)
                let seed = read_wav(p)?.fields;
                set_id3(&mut t, &seed);
            }
            set_id3(&mut t, changes);
            let final_fields = fields_from_id3(&t);
            let pairs: Vec<(&str, String)> = INFO_MAP
                .iter()
                .map(|(field, id)| (*id, final_fields.get(*field).cloned().unwrap_or_default()))
                .collect();
            let refs: Vec<(&str, &str)> = pairs.iter().map(|(a, b)| (*a, b.as_str())).chain([("IPRT", "")]).collect();
            let mut buf = Vec::new();
            t.write_to(&mut buf, id3_version(&t))?;
            riff::write_wav(p, &refs, Some(&buf))?;
            Ok(())
        }
        "flac" => write_flac(p, changes),
        "m4a" | "mp4" => write_mp4(p, changes),
        "wma" => bail!("Không hỗ trợ sửa tag WMA"),
        _ => write_generic(p, changes),
    }
}

/// Fast path for in-place writes: FLAC tags that fit the existing metadata area are
/// written without touching the audio (a few KB instead of the whole file over SMB).
pub fn write_track_in_place(p: &Path, changes: &Fields) -> Result<()> {
    if ext(p) == "flac" && changes.keys().all(|k| FIELDS.contains(&k.as_str())) && flac_inplace::write(p, changes)? {
        return Ok(());
    }
    write_track(p, changes)
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

fn set_id3(t: &mut id3::Tag, c: &Fields) {
    use id3::TagLike;
    let text = |t: &mut id3::Tag, id: &str, v: &str| if v.is_empty() { t.remove(id); } else { t.set_text(id, v); };
    let pair = |n: Option<&String>, cur_n: Option<u32>, tot: Option<&String>, cur_t: Option<u32>| -> String {
        let n = n.cloned().unwrap_or_else(|| cur_n.map(|x| x.to_string()).unwrap_or_default());
        let tt = tot.cloned().unwrap_or_else(|| cur_t.map(|x| x.to_string()).unwrap_or_default());
        match (n.is_empty(), tt.is_empty()) {
            (true, _) => String::new(),
            (false, true) => n,
            (false, false) => format!("{n}/{tt}"),
        }
    };
    for (k, v) in c {
        match k.as_str() {
            "title" => text(t, "TIT2", v),
            "artist" => text(t, "TPE1", v),
            "album" => text(t, "TALB", v),
            "composer" => text(t, "TCOM", v),
            "genre" => text(t, "TCON", v),
            "albumartist" => {
                text(t, "TPE2", v);
                // foobar-style duplicate kept in step
                let descs: Vec<String> = t.extended_texts().filter(|x| x.description.eq_ignore_ascii_case("album artist")).map(|x| x.description.clone()).collect();
                for d in descs {
                    t.remove_extended_text(Some(&d), None);
                    if !v.is_empty() {
                        t.add_frame(id3::frame::ExtendedText { description: d, value: v.clone() });
                    }
                }
            }
            "year" => {
                let (keep, drop) = if t.version() == id3::Version::Id3v24 { ("TDRC", "TYER") } else { ("TYER", "TDRC") };
                t.remove(drop);
                text(t, keep, v);
            }
            "comment" => {
                t.remove_comment(Some(""), None);
                if !v.is_empty() {
                    t.add_frame(id3::frame::Comment { lang: "eng".into(), description: String::new(), text: v.clone() });
                }
            }
            _ => {}
        }
    }
    if c.contains_key("track") || c.contains_key("tracktotal") {
        let s = pair(c.get("track"), t.track(), c.get("tracktotal"), t.total_tracks());
        text(t, "TRCK", &s);
    }
    if c.contains_key("disc") || c.contains_key("disctotal") {
        let s = pair(c.get("disc"), t.disc(), c.get("disctotal"), t.total_discs());
        text(t, "TPOS", &s);
    }
}

fn write_flac(p: &Path, c: &Fields) -> Result<()> {
    let mut f = File::open(p)?;
    let mut ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new())?;
    drop(f);
    if ff.vorbis_comments().is_none() {
        ff.set_vorbis_comments(lofty::ogg::tag::VorbisComments::default());
    }
    let vc = ff.vorbis_comments_mut().unwrap();
    for (field, keys) in VORBIS_KEYS {
        let Some(v) = c.get(*field) else { continue };
        // update every spelling the file already uses, and always the standard key
        // (players such as Roon look for ALBUMARTIST, not "ALBUM ARTIST")
        let mut targets: Vec<String> = vec![keys[0].to_string()];
        for k in vc.items().map(|(k, _)| k.to_string()) {
            if keys.iter().any(|x| x.eq_ignore_ascii_case(&k)) && !targets.iter().any(|t| t.eq_ignore_ascii_case(&k)) {
                targets.push(k);
            }
        }
        for k in targets {
            let _ = vc.remove(&k).count();
            if !v.is_empty() {
                vc.push(k, v.clone());
            }
        }
    }
    // a "3/12" style number would contradict a new total: store the plain number
    for (n, tot, key) in [("track", "tracktotal", "TRACKNUMBER"), ("disc", "disctotal", "DISCNUMBER")] {
        if c.contains_key(tot) && !c.contains_key(n) {
            let cur: Option<String> = vc.items().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.to_string());
            if let Some(cur) = cur.filter(|v| v.contains('/')) {
                let _ = vc.remove(key).count();
                vc.push(key.to_string(), split_slash(&cur).0);
            }
        }
    }
    ff.save_to_path(p, WriteOptions::default())?;
    Ok(())
}

fn write_mp4(p: &Path, c: &Fields) -> Result<()> {
    use lofty::mp4::{Atom, AtomData, AtomIdent};
    let mut f = File::open(p)?;
    let mut mf = lofty::mp4::Mp4File::read_from(&mut f, ParseOptions::new())?;
    drop(f);
    if mf.ilst().is_none() {
        mf.set_ilst(lofty::mp4::Ilst::default());
    }
    let il = mf.ilst_mut().unwrap();
    let atoms: &[(&str, [u8; 4])] = &[
        ("title", *b"\xa9nam"), ("artist", *b"\xa9ART"), ("album", *b"\xa9alb"), ("albumartist", *b"aART"),
        ("composer", *b"\xa9wrt"), ("genre", *b"\xa9gen"), ("year", *b"\xa9day"), ("comment", *b"\xa9cmt"),
    ];
    for (field, code) in atoms {
        let Some(v) = c.get(*field) else { continue };
        let id = AtomIdent::Fourcc(*code);
        let _ = il.remove(&id).count();
        if *field == "genre" {
            let _ = il.remove(&AtomIdent::Fourcc(*b"gnre")).count();
        }
        if !v.is_empty() {
            il.insert(Atom::new(id, AtomData::UTF8(v.clone())));
        }
    }
    let n = |v: &String| v.parse::<u32>().ok();
    if let Some(v) = c.get("track") { match n(v) { Some(x) => il.set_track(x), None => il.remove_track() } }
    if let Some(v) = c.get("tracktotal") { match n(v) { Some(x) => il.set_track_total(x), None => il.remove_track_total() } }
    if let Some(v) = c.get("disc") { match n(v) { Some(x) => il.set_disk(x), None => il.remove_disk() } }
    if let Some(v) = c.get("disctotal") { match n(v) { Some(x) => il.set_disk_total(x), None => il.remove_disk_total() } }
    mf.save_to_path(p, WriteOptions::default())?;
    Ok(())
}

/// APE, Ogg, Opus, WavPack: lofty's generic tag kept every item in the spike (APE).
fn write_generic(p: &Path, c: &Fields) -> Result<()> {
    let mut tf = lofty::read_from_path(p)?;
    let ty = tf.primary_tag_type();
    if tf.tag(ty).is_none() {
        tf.insert_tag(Tag::new(ty));
    }
    let t = tf.tag_mut(ty).unwrap();
    let keys: &[(&str, ItemKey)] = &[
        ("title", ItemKey::TrackTitle), ("artist", ItemKey::TrackArtist), ("album", ItemKey::AlbumTitle),
        ("albumartist", ItemKey::AlbumArtist), ("composer", ItemKey::Composer), ("genre", ItemKey::Genre),
        ("year", ItemKey::Year), ("comment", ItemKey::Comment), ("track", ItemKey::TrackNumber),
        ("tracktotal", ItemKey::TrackTotal), ("disc", ItemKey::DiscNumber), ("disctotal", ItemKey::DiscTotal),
    ];
    for (field, key) in keys {
        if let Some(v) = c.get(*field) {
            if v.is_empty() {
                t.remove_key(key.clone());
            } else {
                t.insert_text(key.clone(), v.clone());
            }
        }
    }
    let _ = TagType::Id3v2;
    tf.save_to_path(p, WriteOptions::default())?;
    Ok(())
}

/// Read the file back and check every written field (APP-WRITE-R6).
pub fn verify(p: &Path, changes: &Fields) -> Result<()> {
    let got = read_track(p)?.fields;
    for (k, v) in changes {
        let want = canonical(k, v);
        let have = canonical(k, got.get(k).map(String::as_str).unwrap_or(""));
        if want != have {
            bail!("Giá trị sau khi ghi không khớp ({k}: «{have}» thay vì «{want}»)");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
