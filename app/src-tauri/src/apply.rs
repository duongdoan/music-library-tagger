//! Apply runs (APP-WRITE-R1..R11) and undo plans (APP-UNDO-R1..R5).
use crate::db::Db;
use crate::model::{Fields, StageInput, TrackRow};
use crate::norm::canonical;
use crate::scan::{mtime_of, read_row};
use crate::tags;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Staged pseudo-field: rewrite a WAV only to re-sync RIFF INFO (APP-EDIT-RIFFSYNC).
pub const RIFF_SYNC: &str = "_riffsync";
/// Staged pseudo-field: rewrite Album Artist so every key the file uses agrees (APP-TAG-R13).
pub const AA_SYNC: &str = "_aasync";
/// Files above this size are written in place (journal still protects them).
pub const ATOMIC_LIMIT: u64 = 500 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyItem {
    pub path: String,
    pub field: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub path: String,
    /// "ok" | "conflict" | "error" | "cancelled"
    pub status: String,
    pub message: Option<String>,
    pub row: Option<TrackRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub run_id: i64,
    pub ok: usize,
    pub conflict: usize,
    pub error: usize,
    pub cancelled: usize,
    pub results: Vec<FileResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoPlan {
    pub run_id: i64,
    /// changes to stage so the run is reversed
    pub stage: Vec<StageInput>,
    /// fields edited again after the run: not staged by default (APP-UNDO-R4)
    pub changed_after: Vec<ChangedAfter>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedAfter {
    pub path: String,
    pub field: String,
    pub current: String,
    pub before: String,
}

fn tmp_path(p: &Path) -> PathBuf {
    // keep the real extension last: format detection relies on it (spike-report §1.4)
    p.with_file_name(format!(".mlt-tmp.{}", p.file_name().unwrap().to_string_lossy()))
}

/// Write via a temp copy in the same folder, verify, then rename over the original
/// (APP-WRITE-R5, R6). On any failure the original is untouched and the temp is removed.
pub fn write_safely(p: &Path, changes: &Fields, in_place: bool) -> Result<()> {
    let size = fs::metadata(p)?.len();
    let real: Fields = changes.iter().filter(|(k, _)| !k.starts_with('_')).map(|(k, v)| (k.clone(), v.clone())).collect();
    if in_place || size > ATOMIC_LIMIT {
        tags::write_track_in_place(p, &real)?;
        return tags::verify(p, &real);
    }
    let tmp = tmp_path(p);
    let r = (|| -> Result<()> {
        fs::copy(p, &tmp)?;
        tags::write_track(&tmp, &real)?;
        tags::verify(&tmp, &real)?;
        fs::rename(&tmp, p)?;
        Ok(())
    })();
    if r.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    r
}

fn friendly(e: &anyhow::Error) -> String {
    if let Some(io) = e.downcast_ref::<std::io::Error>() {
        return match io.kind() {
            std::io::ErrorKind::PermissionDenied => "Không có quyền ghi file".into(),
            std::io::ErrorKind::NotFound => "File không còn tồn tại".into(),
            _ => format!("Lỗi ghi: {io}"),
        };
    }
    let s = format!("{e:#}");
    if s.contains("Permission denied") || s.contains("Read-only") { "Không có quyền ghi file".into() } else { s }
}

fn label(field: &str) -> &str {
    match field {
        "title" => "Title", "artist" => "Artist", "album" => "Album", "albumartist" => "Album Artist", "composer" => "Composer",
        "genre" => "Genre", "year" => "Year", "track" => "Track", "tracktotal" => "Track Total", "disc" => "Disc",
        "disctotal" => "Disc Total", "comment" => "Comment", RIFF_SYNC => "RIFF INFO", AA_SYNC => "Khoá Album Artist", other => other,
    }
}

/// Apply the selected staged changes, one file at a time (APP-WRITE-R10: sequential
/// is the safe default on a network share).
pub fn apply(
    db: &Mutex<Db>,
    items: &[ApplyItem],
    undo_of: Option<i64>,
    in_place: bool,
    cancel: &AtomicBool,
    on_file: &(dyn Fn(usize, usize, &FileResult) + Sync),
) -> Result<Report> {
    let staged: HashMap<(String, String), String> = db.lock().unwrap().staged()?.into_iter().map(|s| ((s.path, s.field), s.value)).collect();
    let mut by_file: BTreeMap<String, Fields> = BTreeMap::new();
    for it in items {
        if let Some(v) = staged.get(&(it.path.clone(), it.field.clone())) {
            by_file.entry(it.path.clone()).or_default().insert(it.field.clone(), v.clone());
        }
    }
    let label_txt = undo_of.map(|r| format!("Hoàn tác của lượt {r}"));
    let kind = if undo_of.is_some() { "undo" } else { "tags" };
    let run = db.lock().unwrap().start_run(kind, label_txt.as_deref(), undo_of)?;
    let total = by_file.len();
    let mut report = Report { run_id: run, ok: 0, conflict: 0, error: 0, cancelled: 0, results: vec![] };
    let mut per_field: BTreeMap<String, usize> = BTreeMap::new();

    for (i, (path, changes)) in by_file.iter().enumerate() {
        if !cancel.load(Ordering::Relaxed) {
            // tell the UI which file is being written (a network write can take seconds)
            on_file(i, total, &FileResult { path: path.clone(), status: "writing".into(), message: None, row: None });
        }
        let res = if cancel.load(Ordering::Relaxed) {
            FileResult { path: path.clone(), status: "cancelled".into(), message: None, row: None }
        } else {
            match apply_one(db, run, path, changes, in_place) {
                Ok(Outcome::Ok(row)) => {
                    for k in changes.keys() {
                        *per_field.entry(k.clone()).or_default() += 1;
                    }
                    FileResult { path: path.clone(), status: "ok".into(), message: None, row: Some(row) }
                }
                Ok(Outcome::Conflict(row)) => FileResult {
                    path: path.clone(),
                    status: "conflict".into(),
                    message: Some("File đã bị thay đổi bên ngoài app".into()),
                    row,
                },
                Err(Fatal::Journal(e)) => {
                    // APP-WRITE-R3: without a backup nothing more is written
                    db.lock().unwrap().finish_run(run, "failed", report.ok as i64, report.conflict as i64, report.error as i64 + 1, "")?;
                    return Err(anyhow!("Không thể sao lưu tag gốc, đã dừng ghi: {e:#}"));
                }
                Err(Fatal::File(e)) => FileResult { path: path.clone(), status: "error".into(), message: Some(friendly(&e)), row: None },
            }
        };
        match res.status.as_str() {
            "ok" => report.ok += 1,
            "conflict" => report.conflict += 1,
            "error" => report.error += 1,
            _ => report.cancelled += 1,
        }
        on_file(i + 1, total, &res);
        report.results.push(res);
    }
    let summary = per_field.iter().map(|(k, n)| format!("{}: {n} file", label(k))).collect::<Vec<_>>().join(", ");
    let status = if report.cancelled > 0 { "cancelled" } else if report.conflict + report.error > 0 { "partial" } else { "done" };
    let db = db.lock().unwrap();
    db.finish_run(run, status, report.ok as i64, report.conflict as i64, report.error as i64, &summary)?;
    if let (Some(orig), true) = (undo_of, report.ok > 0) {
        // APP-UNDO: mark the original run; partial when something could not be reversed
        let st = if report.conflict + report.error + report.cancelled == 0 { "undone" } else { "partially_undone" };
        db.conn.execute("UPDATE runs SET status = ?2 WHERE id = ?1", rusqlite::params![orig, st])?;
    }
    db.prune_runs()?;
    Ok(report)
}

enum Outcome {
    Ok(TrackRow),
    Conflict(Option<TrackRow>),
}

enum Fatal {
    Journal(anyhow::Error),
    File(anyhow::Error),
}

fn apply_one(db: &Mutex<Db>, run: i64, path: &str, changes: &Fields, in_place: bool) -> std::result::Result<Outcome, Fatal> {
    let p = Path::new(path);
    let indexed = db.lock().unwrap().track(path).map_err(Fatal::File)?.ok_or_else(|| Fatal::File(anyhow!("File không có trong thư viện")))?;
    let meta = fs::metadata(p).map_err(|e| Fatal::File(e.into()))?;
    // APP-WRITE-R4: changed outside the app since the scan
    if meta.len() != indexed.size || mtime_of(&meta) != indexed.mtime {
        let fresh = read_row(indexed.source_id, p, meta.len(), mtime_of(&meta));
        return Ok(Outcome::Conflict(Some(fresh)));
    }
    // APP-WRITE-R3: journal the values the file holds right now
    let current = tags::read_track(p).map_err(Fatal::File)?.fields;
    let mut changes = changes.clone();
    if changes.contains_key(AA_SYNC) && !changes.contains_key("albumartist") {
        // writing the shown value again updates every Album Artist key the file uses
        changes.insert("albumartist".into(), current.get("albumartist").cloned().unwrap_or_default());
    }
    let changes = &changes;
    let entries: Vec<(String, String, String)> = changes
        .iter()
        .map(|(k, v)| (k.clone(), if k.starts_with('_') { String::new() } else { current.get(k).cloned().unwrap_or_default() }, v.clone()))
        .collect();
    db.lock().unwrap().journal_before(run, path, &entries).map_err(Fatal::Journal)?;

    write_safely(p, changes, in_place).map_err(Fatal::File)?;

    let m2 = fs::metadata(p).map_err(|e| Fatal::File(e.into()))?;
    let row = read_row(indexed.source_id, p, m2.len(), mtime_of(&m2));
    let keys: Vec<(String, String)> = changes.keys().map(|k| (path.to_string(), k.clone())).collect();
    let db = db.lock().unwrap();
    db.upsert_tracks(std::slice::from_ref(&row)).map_err(Fatal::File)?;
    db.discard(&keys).map_err(Fatal::File)?;
    db.journal_ok(run, path).map_err(Fatal::File)?;
    Ok(Outcome::Ok(row))
}

/// Reverse a run as staged changes (APP-UNDO-R2..R4).
pub fn undo_plan(db: &Db, run: i64) -> Result<UndoPlan> {
    let mut plan = UndoPlan { run_id: run, stage: vec![], changed_after: vec![], missing: vec![] };
    for (path, field, before, after) in db.journal(run)? {
        if field.starts_with('_') {
            continue;
        }
        let Some(t) = db.track(&path)? else {
            if !plan.missing.contains(&path) {
                plan.missing.push(path);
            }
            continue;
        };
        let cur = t.fields.get(&field).cloned().unwrap_or_default();
        if canonical(&field, &cur) == canonical(&field, &after) {
            plan.stage.push(StageInput { path, field, value: before });
        } else {
            plan.changed_after.push(ChangedAfter { path, field, current: cur, before });
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan;

    fn wav(p: &Path, title: &str) {
        let mut b = b"RIFF\0\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x44\xac\0\0\x88\x58\x01\0\x02\0\x10\0data\x04\0\0\0\x01\x02\x03\x04".to_vec();
        let n = (b.len() - 8) as u32;
        b[4..8].copy_from_slice(&n.to_le_bytes());
        fs::write(p, b).unwrap();
        let mut f = Fields::new();
        f.insert("title".into(), title.into());
        tags::write_track(p, &f).unwrap();
    }

    fn setup(n: usize) -> (tempfile::TempDir, Mutex<Db>, Vec<String>) {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("lib");
        fs::create_dir_all(&root).unwrap();
        let mut paths = vec![];
        for i in 0..n {
            let p = root.join(format!("{i}.wav"));
            wav(&p, &format!("T{i}"));
            paths.push(p.to_string_lossy().to_string());
        }
        let db = Mutex::new(Db::open_memory().unwrap());
        let src = db.lock().unwrap().add_source(&root.to_string_lossy(), "lib").unwrap();
        scan::scan(&db, &src, &root, &AtomicBool::new(false), 2, &|_| {}, &|_, _| {}).unwrap();
        (d, db, paths)
    }

    fn stage(db: &Mutex<Db>, path: &str, field: &str, v: &str) {
        db.lock().unwrap().stage_set(&[(path.into(), field.into(), v.into())]).unwrap();
    }

    fn items(db: &Mutex<Db>) -> Vec<ApplyItem> {
        db.lock().unwrap().staged().unwrap().into_iter().map(|s| ApplyItem { path: s.path, field: s.field }).collect()
    }

    #[test]
    fn apply_writes_journals_and_clears_staged() {
        let (_d, db, p) = setup(2);
        stage(&db, &p[0], "album", "Bolero Tuyển Chọn IV");
        stage(&db, &p[1], "album", "Bolero Tuyển Chọn IV");
        let r = apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        assert_eq!((r.ok, r.conflict, r.error), (2, 0, 0));
        assert!(db.lock().unwrap().staged().unwrap().is_empty());
        assert_eq!(tags::read_track(Path::new(&p[0])).unwrap().fields.get("album").unwrap(), "Bolero Tuyển Chọn IV");
        let runs = db.lock().unwrap().runs().unwrap();
        assert_eq!(runs[0].summary, "Album: 2 file");
        assert!(!Path::new(&p[0]).with_file_name(".mlt-tmp.0.wav").exists());
    }

    #[test]
    fn external_change_is_a_conflict_and_file_untouched() {
        let (_d, db, p) = setup(1);
        stage(&db, &p[0], "title", "Mine");
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut f = Fields::new();
        f.insert("artist".into(), "Someone else".into());
        tags::write_track(Path::new(&p[0]), &f).unwrap(); // another app edits the file
        let r = apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        assert_eq!(r.conflict, 1);
        assert_eq!(tags::read_track(Path::new(&p[0])).unwrap().fields.get("title").unwrap(), "T0");
        assert_eq!(db.lock().unwrap().staged().unwrap().len(), 1, "staged change kept");
    }

    #[test]
    fn read_only_file_is_an_error_and_others_continue() {
        let (_d, db, p) = setup(2);
        stage(&db, &p[0], "title", "X");
        stage(&db, &p[1], "title", "Y");
        let dir = Path::new(&p[0]).parent().unwrap().to_path_buf();
        // make the folder read-only: the temp copy cannot be created
        let mut perm = fs::metadata(&dir).unwrap().permissions();
        use std::os::unix::fs::PermissionsExt;
        perm.set_mode(0o555);
        fs::set_permissions(&dir, perm.clone()).unwrap();
        let r = apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        perm.set_mode(0o755);
        fs::set_permissions(&dir, perm).unwrap();
        assert_eq!(r.error, 2);
        assert_eq!(r.results[0].message.as_deref(), Some("Không có quyền ghi file"));
        assert_eq!(db.lock().unwrap().staged().unwrap().len(), 2);
    }

    #[test]
    fn album_artist_key_sync_rewrites_the_shown_value() {
        let (_d, db, p) = setup(1);
        // give the file a TXXX:Album Artist that disagrees with TPE2
        let path = Path::new(&p[0]);
        let mut t = id3::Tag::read_from_path(path).unwrap();
        use id3::TagLike;
        t.set_album_artist("Kyung Wha Chung");
        t.add_frame(id3::frame::ExtendedText { description: "Album Artist".into(), value: "".into() });
        let mut buf = vec![];
        t.write_to(&mut buf, id3::Version::Id3v24).unwrap();
        crate::tags::riff::write_wav(path, &[], Some(&buf)).unwrap();
        let src = db.lock().unwrap().sources().unwrap()[0].clone();
        scan::scan(&db, &src, path.parent().unwrap(), &AtomicBool::new(false), 1, &|_| {}, &|_, _| {}).unwrap();
        assert!(db.lock().unwrap().track(&p[0]).unwrap().unwrap().aa_mismatch);
        stage(&db, &p[0], AA_SYNC, "1");
        let r = apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        assert_eq!(r.ok, 1, "{:?}", r.results);
        let row = db.lock().unwrap().track(&p[0]).unwrap().unwrap();
        assert!(!row.aa_mismatch);
        assert_eq!(row.fields.get("albumartist").unwrap(), "Kyung Wha Chung");
    }

    #[test]
    fn cancel_leaves_rest_staged() {
        let (_d, db, p) = setup(3);
        for x in &p {
            stage(&db, x, "genre", "Jazz");
        }
        let cancel = AtomicBool::new(false);
        let r = apply(&db, &items(&db), None, false, &cancel, &|i, _, _| if i == 1 { cancel.store(true, Ordering::Relaxed) }).unwrap();
        assert_eq!((r.ok, r.cancelled), (1, 2));
        assert_eq!(db.lock().unwrap().staged().unwrap().len(), 2);
    }

    #[test]
    fn undo_plan_reverses_and_protects_later_edits() {
        let (_d, db, p) = setup(2);
        stage(&db, &p[0], "title", "A1");
        stage(&db, &p[1], "title", "B1");
        let r1 = apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        // a later run edits file 1 again
        stage(&db, &p[1], "title", "B2");
        apply(&db, &items(&db), None, false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        let plan = undo_plan(&db.lock().unwrap(), r1.run_id).unwrap();
        assert_eq!(plan.stage.len(), 1);
        assert_eq!(plan.stage[0].value, "T0");
        assert_eq!(plan.changed_after.len(), 1, "APP-UNDO-R4");
        assert_eq!(plan.changed_after[0].current, "B2");
        // stage + apply the undo
        let db2 = db.lock().unwrap();
        db2.stage_set(&plan.stage.iter().map(|s| (s.path.clone(), s.field.clone(), s.value.clone())).collect::<Vec<_>>()).unwrap();
        drop(db2);
        let r = apply(&db, &items(&db), Some(r1.run_id), false, &AtomicBool::new(false), &|_, _, _| {}).unwrap();
        assert_eq!(r.ok, 1);
        assert_eq!(tags::read_track(Path::new(&p[0])).unwrap().fields.get("title").unwrap(), "T0");
        assert_eq!(db.lock().unwrap().runs().unwrap()[0].label.as_deref(), Some("Hoàn tác của lượt 1"));
    }
}
