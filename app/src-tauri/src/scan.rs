//! Library scan (APP-SCAN-R1..R12): parallel listing, incremental by size + mtime,
//! saved every batch so an interrupted first scan resumes where it stopped.
use crate::db::Db;
use crate::model::{Source, TrackRow, AUDIO_EXT, LIST_ONLY_EXT};
use crate::tags;
use anyhow::Result;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Instant, UNIX_EPOCH};

pub const BATCH: usize = 200;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub source_id: i64,
    /// "list" | "read"
    pub phase: String,
    pub done: usize,
    pub total: usize,
    pub per_sec: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub source_id: i64,
    pub read: usize,
    pub added: usize,
    pub changed: usize,
    pub removed: usize,
    pub errors: usize,
    pub skipped: usize,
    pub cancelled: bool,
    pub unavailable: bool,
}

struct Entry {
    path: PathBuf,
    size: u64,
    mtime: i64,
}

pub fn mtime_of(m: &fs::Metadata) -> i64 {
    m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_nanos() as i64).unwrap_or(0)
}

fn excluded(name: &str, excludes: &[String]) -> bool {
    // hidden files and macOS AppleDouble files are never scanned (APP-SCAN-R1)
    name.starts_with('.') || excludes.iter().any(|e| e == name)
}

fn walk(dir: &Path, excludes: &[String], out: &Mutex<Vec<Entry>>, skipped: &AtomicUsize, listed: &AtomicUsize, cancel: &AtomicBool) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut subdirs = vec![];
    let mut local = vec![];
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if excluded(&name, excludes) {
            continue;
        }
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            subdirs.push(e.path());
            continue;
        }
        let p = e.path();
        if !AUDIO_EXT.contains(&tags::ext(&p).as_str()) {
            skipped.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        if let Ok(m) = e.metadata() {
            local.push(Entry { path: p, size: m.len(), mtime: mtime_of(&m) });
        }
    }
    listed.fetch_add(local.len(), Ordering::Relaxed);
    out.lock().unwrap().extend(local);
    subdirs.par_iter().for_each(|d| walk(d, excludes, out, skipped, listed, cancel));
}

pub fn read_row(source_id: i64, path: &Path, size: u64, mtime: i64) -> TrackRow {
    let ext = tags::ext(path);
    let mut row = TrackRow {
        path: path.to_string_lossy().to_string(),
        source_id,
        dir: path.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default(),
        file: path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default(),
        ext: ext.clone(),
        size,
        mtime,
        fields: Default::default(),
        has_art: false,
        format: ext.to_uppercase(),
        duration_ms: None,
        riff_mismatch: false,
        aa_mismatch: false,
        error: None,
        readonly: false,
    };
    if LIST_ONLY_EXT.contains(&ext.as_str()) {
        row.readonly = true; // APP-TAG-R11
        return row;
    }
    match tags::read_track(path) {
        Ok(t) => {
            row.fields = t.fields;
            row.has_art = t.has_art;
            row.format = t.format;
            row.duration_ms = t.duration_ms;
            row.riff_mismatch = t.riff_mismatch;
            row.aa_mismatch = t.aa_mismatch;
        }
        Err(e) => row.error = Some(format!("Lỗi đọc: {e}")),
    }
    row
}

/// Scan `root` (the source root or one folder inside it, APP-SCAN-R12).
pub fn scan(
    db: &Mutex<Db>,
    source: &Source,
    root: &Path,
    cancel: &AtomicBool,
    threads: usize,
    on_progress: &(dyn Fn(Progress) + Sync),
    on_batch: &(dyn Fn(Vec<TrackRow>, Vec<String>) + Sync),
) -> Result<Summary> {
    let mut sum = Summary { source_id: source.id, ..Default::default() };
    if !Path::new(&source.path).is_dir() || !root.is_dir() {
        // APP-SCAN-R7: keep the index, mark the source unavailable
        db.lock().unwrap().set_source_status(source.id, "unavailable", false)?;
        sum.unavailable = true;
        return Ok(sum);
    }
    db.lock().unwrap().set_source_status(source.id, "scanning", false)?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
    let started = Instant::now();

    // 1. list
    let entries = Mutex::new(Vec::new());
    let skipped = AtomicUsize::new(0);
    let listed = AtomicUsize::new(0);
    let done = AtomicBool::new(false);
    std::thread::scope(|s| {
        s.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                on_progress(Progress { source_id: source.id, phase: "list".into(), done: listed.load(Ordering::Relaxed), total: 0, per_sec: 0.0 });
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        });
        pool.install(|| walk(root, &source.excludes, &entries, &skipped, &listed, cancel));
        done.store(true, Ordering::Relaxed);
    });
    sum.skipped = skipped.load(Ordering::Relaxed);
    let entries = entries.into_inner().unwrap();
    if cancel.load(Ordering::Relaxed) {
        sum.cancelled = true;
        db.lock().unwrap().set_source_status(source.id, "cancelled", false)?;
        return Ok(sum);
    }

    // 2. diff with the index
    let mut prefix = root.to_string_lossy().to_string();
    if !prefix.ends_with('/') {
        prefix.push('/');
    }
    let stamps = db.lock().unwrap().stamps(source.id, &prefix)?;
    let on_disk: HashSet<String> = entries.iter().map(|e| e.path.to_string_lossy().to_string()).collect();
    let removed: Vec<String> = stamps.keys().filter(|p| !on_disk.contains(*p)).cloned().collect();
    let todo: Vec<&Entry> = entries
        .iter()
        .filter(|e| stamps.get(&*e.path.to_string_lossy()).map(|&(s, m)| s != e.size || m != e.mtime).unwrap_or(true))
        .collect();
    let total = todo.len();
    let t_read = Instant::now();

    // 3. read in batches, saving each batch
    for chunk in todo.chunks(BATCH) {
        if cancel.load(Ordering::Relaxed) {
            sum.cancelled = true;
            break;
        }
        let rows: Vec<TrackRow> = pool.install(|| chunk.par_iter().map(|e| read_row(source.id, &e.path, e.size, e.mtime)).collect());
        for r in &rows {
            if stamps.contains_key(&r.path) { sum.changed += 1 } else { sum.added += 1 }
            if r.error.is_some() { sum.errors += 1 }
        }
        sum.read += rows.len();
        db.lock().unwrap().upsert_tracks(&rows)?;
        on_batch(rows, vec![]);
        let secs = t_read.elapsed().as_secs_f64().max(0.001);
        on_progress(Progress { source_id: source.id, phase: "read".into(), done: sum.read, total, per_sec: sum.read as f64 / secs });
    }

    // 4. drop files that are gone (only when the listing was complete)
    if !sum.cancelled && !removed.is_empty() {
        db.lock().unwrap().delete_tracks(&removed)?;
        sum.removed = removed.len();
        on_batch(vec![], removed);
    }
    let status = if sum.cancelled { "cancelled" } else { "ready" };
    db.lock().unwrap().set_source_status(source.id, status, !sum.cancelled)?;
    let _ = started;
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(p: &Path) {
        let mut b = b"RIFF\0\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x44\xac\0\0\x88\x58\x01\0\x02\0\x10\0data\x04\0\0\0\0\0\0\0".to_vec();
        let n = (b.len() - 8) as u32;
        b[4..8].copy_from_slice(&n.to_le_bytes());
        fs::write(p, b).unwrap();
    }

    fn run(db: &Mutex<Db>, src: &Source) -> Summary {
        let cancel = AtomicBool::new(false);
        scan(db, src, Path::new(&src.path), &cancel, 4, &|_| {}, &|_, _| {}).unwrap()
    }

    #[test]
    fn incremental_scan_reads_only_changes() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("lib");
        fs::create_dir_all(root.join("A/CD1")).unwrap();
        fs::create_dir_all(root.join(".hidden")).unwrap();
        fs::create_dir_all(root.join("roon-backup")).unwrap();
        for i in 0..5 { wav(&root.join(format!("A/CD1/{i}.wav"))); }
        wav(&root.join(".hidden/x.wav"));
        wav(&root.join("roon-backup/y.wav"));
        wav(&root.join("A/._junk.wav"));
        fs::write(root.join("A/cover.jpg"), b"jpg").unwrap();
        fs::write(root.join("A/broken.flac"), b"not a flac").unwrap();

        let db = Mutex::new(Db::open_memory().unwrap());
        let src = db.lock().unwrap().add_source(&root.to_string_lossy(), "lib").unwrap();
        db.lock().unwrap().set_excludes(src.id, &["roon-backup".into()]).unwrap();
        let src = db.lock().unwrap().source(src.id).unwrap().unwrap();

        let s = run(&db, &src);
        assert_eq!((s.read, s.added, s.errors, s.skipped), (6, 6, 1, 1), "{s:?}");

        // second scan: nothing changed
        let s = run(&db, &src);
        assert_eq!(s.read, 0);

        // change one, delete one, add one
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut b = fs::read(root.join("A/CD1/0.wav")).unwrap();
        b.extend([0, 0]);
        let n = (b.len() - 8) as u32;
        b[4..8].copy_from_slice(&n.to_le_bytes());
        fs::write(root.join("A/CD1/0.wav"), b).unwrap();
        fs::remove_file(root.join("A/CD1/1.wav")).unwrap();
        wav(&root.join("A/CD1/9.wav"));
        let s = run(&db, &src);
        assert_eq!((s.read, s.changed, s.added, s.removed), (2, 1, 1, 1), "{s:?}");
        assert_eq!(db.lock().unwrap().tracks().unwrap().len(), 6);
    }

    #[test]
    fn missing_source_is_unavailable_and_index_kept() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("lib");
        fs::create_dir_all(&root).unwrap();
        wav(&root.join("a.wav"));
        let db = Mutex::new(Db::open_memory().unwrap());
        let src = db.lock().unwrap().add_source(&root.to_string_lossy(), "lib").unwrap();
        run(&db, &src);
        fs::remove_dir_all(&root).unwrap();
        let s = run(&db, &src);
        assert!(s.unavailable);
        assert_eq!(db.lock().unwrap().tracks().unwrap().len(), 1, "APP-SCAN-R7");
        assert_eq!(db.lock().unwrap().source(src.id).unwrap().unwrap().status, "unavailable");
    }
}
