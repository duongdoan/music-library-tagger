use super::*;
use crate::model::Fields;
use std::fs;
use std::path::PathBuf;

fn fields(pairs: &[(&str, &str)]) -> Fields {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn le16(v: u16) -> [u8; 2] { v.to_le_bytes() }
fn le32(v: u32) -> [u8; 4] { v.to_le_bytes() }
fn le64(v: u64) -> [u8; 8] { v.to_le_bytes() }

/// 16-bit stereo 44.1 kHz WAV with 0.05 s of a ramp, optional raw LIST/INFO body.
fn make_wav(dir: &Path, name: &str, info: &[(&[u8; 4], &[u8])]) -> PathBuf {
    let data: Vec<u8> = (0..8820u32).map(|i| (i % 251) as u8).collect();
    let mut fmt = vec![];
    fmt.extend(le16(1)); fmt.extend(le16(2)); fmt.extend(le32(44100)); fmt.extend(le32(44100 * 4)); fmt.extend(le16(4)); fmt.extend(le16(16));
    let mut body = b"WAVE".to_vec();
    body.extend(b"fmt "); body.extend(le32(fmt.len() as u32)); body.extend(&fmt);
    body.extend(b"data"); body.extend(le32(data.len() as u32)); body.extend(&data);
    if !info.is_empty() {
        let mut l = b"INFO".to_vec();
        for (id, v) in info {
            let mut v = v.to_vec(); v.push(0);
            l.extend(*id); l.extend(le32(v.len() as u32)); l.extend(&v);
            if v.len() % 2 == 1 { l.push(0); }
        }
        body.extend(b"LIST"); body.extend(le32(l.len() as u32)); body.extend(&l);
    }
    let mut f = b"RIFF".to_vec(); f.extend(le32(body.len() as u32)); f.extend(body);
    let p = dir.join(name);
    fs::write(&p, f).unwrap();
    p
}

/// FLAC with STREAMINFO + VORBIS_COMMENT + a fake 8-byte frame region.
fn make_flac(dir: &Path, comments: &[&str]) -> PathBuf {
    let mut si = vec![0u8; 34];
    si[0..2].copy_from_slice(&4096u16.to_be_bytes()); si[2..4].copy_from_slice(&4096u16.to_be_bytes());
    // 44100 Hz, 2 ch, 16 bit, 441000 samples (10 s)
    let rate: u32 = 44100; let total: u64 = 441_000;
    si[10] = (rate >> 12) as u8; si[11] = (rate >> 4) as u8;
    si[12] = (((rate & 0xf) << 4) as u8) | ((1 << 1) as u8) | 0; // ch-1 = 1, bps-1 high bit = 0
    si[13] = ((15 & 0xf) << 4) as u8 | ((total >> 32) as u8 & 0xf);
    si[14..18].copy_from_slice(&(total as u32).to_be_bytes());
    let mut vc = vec![];
    let vendor = b"test"; vc.extend(le32(vendor.len() as u32)); vc.extend(vendor);
    vc.extend(le32(comments.len() as u32));
    for c in comments { vc.extend(le32(c.len() as u32)); vc.extend(c.as_bytes()); }
    let mut f = b"fLaC".to_vec();
    f.push(0); f.extend(&(si.len() as u32).to_be_bytes()[1..]); f.extend(&si);
    f.push(0x80 | 4); f.extend(&(vc.len() as u32).to_be_bytes()[1..]); f.extend(&vc);
    f.extend([0xff, 0xf8, 0x69, 0x08, 0x00, 0x00, 0x00, 0x00]);
    let p = dir.join("t.flac");
    fs::write(&p, f).unwrap();
    p
}

fn make_dsf(dir: &Path) -> PathBuf {
    let data = vec![0x69u8; 4096 * 2];
    let mut f = b"DSD ".to_vec(); f.extend(le64(28)); f.extend(le64(0)); f.extend(le64(0));
    f.extend(b"fmt "); f.extend(le64(52));
    for v in [1u32, 0, 2, 2, 2_822_400, 1] { f.extend(le32(v)); }
    f.extend(le64(4096 * 8)); f.extend(le32(4096)); f.extend(le32(0));
    f.extend(b"data"); f.extend(le64(12 + data.len() as u64)); f.extend(&data);
    let total = f.len() as u64;
    f[12..20].copy_from_slice(&le64(total));
    let p = dir.join("t.dsf");
    fs::write(&p, f).unwrap();
    p
}

fn audio_hash(p: &Path) -> String { payload::hash(p).unwrap() }

#[test]
fn wav_write_syncs_id3_and_info_and_keeps_other_info_bytes() {
    let d = tempfile::tempdir().unwrap();
    // ISFT holds Windows-1252 bytes that are not UTF-8; it is not a synced field
    let p = make_wav(d.path(), "a.wav", &[(b"INAM", b"Track 1"), (b"IART", b"VINHSTUDIO"), (b"ISFT", b"Si\xeau th\xec")]);
    let before = audio_hash(&p);
    // no ID3 yet: values come from INFO
    let t = read_track(&p).unwrap();
    assert_eq!(t.fields.get("title").unwrap(), "Track 1");
    assert!(!t.riff_mismatch);

    write_track(&p, &fields(&[("title", "Diễm Xưa"), ("artist", "Khánh Ly")])).unwrap();
    verify(&p, &fields(&[("title", "Diễm Xưa"), ("artist", "Khánh Ly")])).unwrap();
    let info = riff::read_info(&p).unwrap();
    let get = |k: &str| info.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
    assert_eq!(get("INAM").as_deref(), Some("Diễm Xưa"));
    assert_eq!(get("IART").as_deref(), Some("Khánh Ly"));
    assert_eq!(get("ISFT").as_deref(), Some("Si\u{ea}u th\u{ec}"), "untouched non-UTF-8 item must survive");
    assert_eq!(audio_hash(&p), before);
    assert!(!read_track(&p).unwrap().riff_mismatch);
}

#[test]
fn wav_mismatch_is_detected_and_fixed_by_empty_write() {
    let d = tempfile::tempdir().unwrap();
    let p = make_wav(d.path(), "b.wav", &[(b"IART", b"VINHSTUDIO LOSSLESS WORLD")]);
    let mut t = id3::Tag::with_version(id3::Version::Id3v23);
    use id3::TagLike;
    t.set_artist("Guns N' Roses");
    t.write_to_wav_path(&p, id3::Version::Id3v23).unwrap();
    let r = read_track(&p).unwrap();
    assert_eq!(r.fields.get("artist").unwrap(), "Guns N' Roses", "ID3 wins (APP-TAG-R10)");
    assert!(r.riff_mismatch);
    write_track(&p, &Fields::new()).unwrap(); // "Đồng bộ RIFF INFO"
    assert!(!read_track(&p).unwrap().riff_mismatch);
}

#[test]
fn flac_fast_reader_and_lossless_write() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["TITLE=Giọt Mưa Thu", "ARTIST=Lệ Thu", "ALBUM ARTIST=Lệ Thu", "TRACKNUMBER=3/12", "UPC=5099909197558", "DATE=1975-01-01"]);
    let t = read_track(&p).unwrap();
    assert_eq!(t.fields.get("title").unwrap(), "Giọt Mưa Thu");
    assert_eq!(t.fields.get("albumartist").unwrap(), "Lệ Thu");
    assert_eq!(t.fields.get("track").unwrap(), "3");
    assert_eq!(t.fields.get("tracktotal").unwrap(), "12");
    assert_eq!(t.fields.get("year").unwrap(), "1975");
    assert_eq!(t.duration_ms, Some(10_000));
    assert_eq!(t.format, "FLAC 16/44.1");

    let ch = fields(&[("albumartist", "Various Artists"), ("tracktotal", "13"), ("genre", "Nhạc vàng")]);
    write_track(&p, &ch).unwrap();
    verify(&p, &ch).unwrap();
    let mut f = File::open(&p).unwrap();
    let ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new()).unwrap();
    let items: Vec<(String, String)> = ff.vorbis_comments().unwrap().items().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    assert!(items.iter().any(|(k, v)| k == "UPC" && v == "5099909197558"), "unknown key kept: {items:?}");
    assert!(items.iter().any(|(k, v)| k == "ALBUM ARTIST" && v == "Various Artists"), "existing spelling updated: {items:?}");
    assert!(items.iter().any(|(k, v)| k == "TRACKNUMBER" && v == "3"), "3/12 rewritten as plain number: {items:?}");
}

#[test]
fn dsf_roundtrip_keeps_audio_and_other_frames() {
    let d = tempfile::tempdir().unwrap();
    let p = make_dsf(d.path());
    let before = audio_hash(&p);
    write_track(&p, &fields(&[("title", "Sapho: Act IV"), ("track", "27"), ("tracktotal", "30")])).unwrap();
    // add an unrelated frame, then change something else
    let mut t = dsf::read_tag(&p).unwrap().unwrap();
    use id3::TagLike;
    t.add_frame(id3::frame::ExtendedText { description: "Dynamic Range (DR)".into(), value: "13".into() });
    dsf::write_tag(&p, &t).unwrap();
    write_track(&p, &fields(&[("artist", "Piunti")])).unwrap();
    let t = dsf::read_tag(&p).unwrap().unwrap();
    assert!(t.extended_texts().any(|x| x.description == "Dynamic Range (DR)" && x.value == "13"));
    let r = read_track(&p).unwrap();
    assert_eq!(r.fields.get("title").unwrap(), "Sapho: Act IV");
    assert_eq!(r.fields.get("track").unwrap(), "27");
    assert_eq!(r.fields.get("tracktotal").unwrap(), "30");
    assert_eq!(r.format, "DSF DSD64");
    assert_eq!(audio_hash(&p), before);
}

#[test]
fn flac_with_two_album_artist_spellings_reads_one_value() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["ALBUMARTIST=A", "ALBUM ARTIST=A", "ARTIST=X", "ARTIST=Y"]);
    let ch = fields(&[("albumartist", "Various Artists")]);
    write_track(&p, &ch).unwrap();
    verify(&p, &ch).unwrap();
    assert_eq!(read_track(&p).unwrap().fields.get("artist").unwrap(), "X; Y");
}

#[test]
fn wav_with_non_riff_trailer_keeps_trailer_and_tags_are_readable() {
    let d = tempfile::tempdir().unwrap();
    let p = make_wav(d.path(), "c.wav", &[]);
    // append an "ID3Helper" style trailer that is not a valid chunk, counted in the RIFF size
    let mut b = fs::read(&p).unwrap();
    let trailer = b"\x00\x09ID3Helper\x001\x00UTF8_Title\x00Ghen\x00".to_vec();
    b.extend(&trailer);
    let n = (b.len() - 8) as u32;
    b[4..8].copy_from_slice(&n.to_le_bytes());
    fs::write(&p, &b).unwrap();
    let ch = fields(&[("title", "Ghen"), ("albumartist", "Various Artists")]);
    write_track(&p, &ch).unwrap();
    verify(&p, &ch).unwrap();
    assert!(fs::read(&p).unwrap().ends_with(&trailer), "trailer bytes kept at the end");
}

#[test]
fn removing_a_field_empties_it() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["TITLE=A", "GENRE=Pop"]);
    write_track(&p, &fields(&[("genre", "")])).unwrap();
    assert!(read_track(&p).unwrap().fields.get("genre").is_none());
}

#[test]
fn rejects_unknown_field_and_wma() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["TITLE=A"]);
    assert!(write_track(&p, &fields(&[("bogus", "x")])).is_err());
    let w = d.path().join("x.wma");
    fs::write(&w, b"0000").unwrap();
    assert!(write_track(&w, &fields(&[("title", "x")])).is_err());
}

/// Real files: `MLT_SAMPLES=/path/to/dir cargo test real_samples -- --nocapture`
#[test]
fn real_samples_roundtrip() {
    let Ok(dir) = std::env::var("MLT_SAMPLES") else { return };
    let tmp = tempfile::tempdir().unwrap();
    for e in fs::read_dir(dir).unwrap().flatten() {
        let src = e.path();
        let x = ext(&src);
        if x == "wma" || !crate::model::AUDIO_EXT.contains(&x.as_str()) { continue; }
        let p = tmp.path().join(src.file_name().unwrap());
        fs::copy(&src, &p).unwrap();
        let orig = read_track(&p).unwrap();
        let h0 = audio_hash(&p);
        let ch = fields(&[("title", "Thử nghiệm"), ("albumartist", "Various Artists"), ("year", "2001")]);
        write_track(&p, &ch).unwrap();
        verify(&p, &ch).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let mut back: Fields = FIELDS.iter().filter(|k| ch.contains_key(**k)).map(|k| (k.to_string(), orig.fields.get(*k).cloned().unwrap_or_default())).collect();
        back.retain(|_, _| true);
        write_track(&p, &back).unwrap();
        let after = read_track(&p).unwrap();
        assert_eq!(after.fields, orig.fields, "{}", p.display());
        assert_eq!(audio_hash(&p), h0, "{}", p.display());
        println!("ok {}", p.file_name().unwrap().to_string_lossy());
    }
}
