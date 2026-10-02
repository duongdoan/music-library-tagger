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
    make_flac_with(dir, comments, &[])
}

/// Same, with extra metadata blocks after the comments: (type, payload).
fn make_flac_with(dir: &Path, comments: &[&str], extra: &[(u8, Vec<u8>)]) -> PathBuf {
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
    f.push(if extra.is_empty() { 0x80 | 4 } else { 4 }); f.extend(&(vc.len() as u32).to_be_bytes()[1..]); f.extend(&vc);
    for (i, (kind, body)) in extra.iter().enumerate() {
        f.push(if i + 1 == extra.len() { 0x80 | kind } else { *kind });
        f.extend(&(body.len() as u32).to_be_bytes()[1..]);
        f.extend(body);
    }
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
    let p = make_flac(d.path(), &["ALBUMARTIST=A", "ALBUM ARTIST=A", "ARTIST=X", "ARTIST=Y", "GENRE=Pop", "GENRE=Rock"]);
    let ch = fields(&[("albumartist", "Various Artists")]);
    write_track(&p, &ch).unwrap();
    verify(&p, &ch).unwrap();
    assert_eq!(read_track(&p).unwrap().fields.get("artist").unwrap(), "X; Y");
    // a key repeated in the file is written once
    write_track(&p, &fields(&[("genre", "Jazz")])).unwrap();
    assert_eq!(read_track(&p).unwrap().fields.get("genre").unwrap(), "Jazz");
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
fn album_artist_key_mismatch_is_flagged_and_fixed_by_writing() {
    let d = tempfile::tempdir().unwrap();
    // the Decca box-set case: empty "albumartist" next to a filled "ALBUM ARTIST"
    let p = make_flac(d.path(), &["album=Walton & Stravinsky - Violin Concertos", "albumartist=", "ALBUM ARTIST=Kyung Wha Chung"]);
    let t = read_track(&p).unwrap();
    assert_eq!(t.fields.get("albumartist").unwrap(), "Kyung Wha Chung");
    assert!(t.aa_mismatch);
    assert_eq!(t.aa_keys, vec![("ALBUMARTIST".to_string(), String::new()), ("ALBUM ARTIST".to_string(), "Kyung Wha Chung".to_string())]);
    write_track(&p, &fields(&[("albumartist", "Kyung Wha Chung")])).unwrap();
    assert!(!read_track(&p).unwrap().aa_mismatch);
    let mut f = File::open(&p).unwrap();
    let ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new()).unwrap();
    let keys: Vec<String> = ff.vorbis_comments().unwrap().items().filter(|(_, v)| *v == "Kyung Wha Chung").map(|(k, _)| k.to_string()).collect();
    assert!(keys.iter().any(|k| k == "ALBUMARTIST"), "standard key written: {keys:?}");
    assert!(keys.iter().any(|k| k == "ALBUM ARTIST"), "existing spelling kept in step: {keys:?}");
    assert_eq!(keys.len(), 2, "no duplicate values: {keys:?}");
    // same value under two keys is fine
    let q = make_flac(d.path(), &["ALBUMARTIST=A", "ALBUM ARTIST=A"]);
    assert!(!read_track(&q).unwrap().aa_mismatch);
    // only the non-standard spelling: flagged, fixed by writing (adds ALBUMARTIST)
    let r = make_flac(d.path(), &["ALBUM ARTIST=A"]);
    assert!(read_track(&r).unwrap().aa_mismatch);
    write_track(&r, &fields(&[("albumartist", "A")])).unwrap();
    assert!(!read_track(&r).unwrap().aa_mismatch);
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

fn picture_block() -> Vec<u8> {
    // minimal PICTURE block: type 3, mime, empty desc, 1x1, 4 bytes of "image"
    let mut b = vec![];
    b.extend(3u32.to_be_bytes());
    b.extend(10u32.to_be_bytes()); b.extend(b"image/jpeg");
    b.extend(0u32.to_be_bytes());
    for v in [1u32, 1, 24, 0] { b.extend(v.to_be_bytes()); }
    b.extend(4u32.to_be_bytes()); b.extend([0xFF, 0xD8, 0xFF, 0xD9]);
    b
}

fn vorbis_items(p: &Path) -> Vec<(String, String)> {
    let mut f = File::open(p).unwrap();
    let ff = lofty::flac::FlacFile::read_from(&mut f, ParseOptions::new()).unwrap();
    ff.vorbis_comments().unwrap().items().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn flac_in_place_uses_padding_and_keeps_audio_and_pictures() {
    let d = tempfile::tempdir().unwrap();
    let pic = picture_block();
    // Decca-style tags, then a picture, then 8 KB padding (typical CUETools/EAC rip)
    let p = make_flac_with(d.path(), &["album=Antar (Mono)", "albumartist=", "ALBUM ARTIST=Ansermet", "TOTALDISCS=54", "UPC=1"], &[(6, pic.clone()), (1, vec![0; 8192])]);
    let size0 = fs::metadata(&p).unwrap().len();
    let audio0 = audio_hash(&p);
    let bytes0 = fs::read(&p).unwrap();
    let ch = fields(&[("albumartist", "L'Orchestre de la Suisse Romande; Ernest Ansermet"), ("disctotal", "")]);
    assert!(flac_inplace::write(&p, &ch).unwrap(), "fits the padding");
    verify(&p, &ch).unwrap();
    assert_eq!(fs::metadata(&p).unwrap().len(), size0, "file size unchanged: written in place");
    assert_eq!(audio_hash(&p), audio0);
    let items = vorbis_items(&p);
    assert!(items.iter().any(|(k, v)| k == "ALBUMARTIST" && v.starts_with("L'Orchestre")), "{items:?}");
    assert!(!items.iter().any(|(k, _)| k.eq_ignore_ascii_case("TOTALDISCS")), "{items:?}");
    assert!(items.iter().any(|(k, v)| k == "UPC" && v == "1"));
    assert!(!read_track(&p).unwrap().aa_mismatch);
    // picture block bytes are still somewhere in the file, untouched
    let bytes1 = fs::read(&p).unwrap();
    assert!(bytes1.windows(pic.len()).any(|w| w == pic.as_slice()));
    assert_eq!(bytes0.len(), bytes1.len());
    // lofty still reads the file and its picture
    let tf = lofty::read_from_path(&p).unwrap();
    assert_eq!(tf.tags().iter().map(|t| t.picture_count()).sum::<u32>(), 1);
}

#[test]
fn flac_in_place_without_room_falls_back() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["TITLE=A"]); // no padding at all
    let big = "x".repeat(500);
    assert!(!flac_inplace::write(&p, &fields(&[("comment", &big)])).unwrap(), "does not fit: nothing written");
    assert_eq!(read_track(&p).unwrap().fields.get("comment"), None);
    write_track_in_place(&p, &fields(&[("comment", &big)])).unwrap(); // full rewrite path
    assert_eq!(read_track(&p).unwrap().fields.get("comment").unwrap(), &big);
}

#[test]
fn flac_in_place_shrink_in_slot_and_same_size() {
    let d = tempfile::tempdir().unwrap();
    let p = make_flac(d.path(), &["TITLE=Long title here", "ARTIST=X"]);
    let size0 = fs::metadata(&p).unwrap().len();
    // shrink by more than 4 bytes: leftover becomes padding in the same slot
    assert!(flac_inplace::write(&p, &fields(&[("title", "Short")])).unwrap());
    assert_eq!(read_track(&p).unwrap().fields.get("title").unwrap(), "Short");
    assert_eq!(fs::metadata(&p).unwrap().len(), size0);
    // grow back into that padding (case: padding right after the comments)
    assert!(flac_inplace::write(&p, &fields(&[("title", "Long title here")])).unwrap());
    assert_eq!(read_track(&p).unwrap().fields.get("title").unwrap(), "Long title here");
    assert_eq!(fs::metadata(&p).unwrap().len(), size0);
}

/// `MLT_SAMPLES=dir cargo test real_flac_in_place -- --nocapture`
#[test]
fn real_flac_in_place() {
    let Ok(dir) = std::env::var("MLT_SAMPLES") else { return };
    let tmp = tempfile::tempdir().unwrap();
    for e in fs::read_dir(dir).unwrap().flatten() {
        if ext(&e.path()) != "flac" { continue; }
        let p = tmp.path().join(e.file_name());
        fs::copy(e.path(), &p).unwrap();
        let (h0, s0) = (audio_hash(&p), fs::metadata(&p).unwrap().len());
        let ch = fields(&[("albumartist", "Kyung Wha Chung, London Symphony Orchestra, André Previn"), ("disctotal", "")]);
        let fast = flac_inplace::write(&p, &ch).unwrap();
        if !fast { write_track(&p, &ch).unwrap(); }
        verify(&p, &ch).unwrap();
        assert_eq!(audio_hash(&p), h0);
        println!("{} in_place={fast} size_same={}", p.file_name().unwrap().to_string_lossy(), fs::metadata(&p).unwrap().len() == s0);
    }
}
