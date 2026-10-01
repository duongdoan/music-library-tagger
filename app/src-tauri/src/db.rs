//! SQLite storage: index, staged changes, apply runs and their journal.
use crate::model::{Fields, RunInfo, Source, StagedChange, TrackRow};
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Db {
    pub conn: Connection,
}

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
CREATE TABLE IF NOT EXISTS sources (
  id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL, name TEXT NOT NULL,
  excludes TEXT NOT NULL DEFAULT '[]', status TEXT NOT NULL DEFAULT 'new', last_scan INTEGER);
CREATE TABLE IF NOT EXISTS tracks (
  path TEXT PRIMARY KEY, source_id INTEGER NOT NULL, dir TEXT NOT NULL, file TEXT NOT NULL, ext TEXT NOT NULL,
  size INTEGER NOT NULL, mtime INTEGER NOT NULL, fields TEXT NOT NULL, has_art INTEGER NOT NULL DEFAULT 0,
  format TEXT NOT NULL DEFAULT '', duration_ms INTEGER, riff_mismatch INTEGER NOT NULL DEFAULT 0,
  error TEXT, readonly INTEGER NOT NULL DEFAULT 0);
CREATE INDEX IF NOT EXISTS tracks_source ON tracks(source_id);
CREATE TABLE IF NOT EXISTS staged (
  path TEXT NOT NULL, field TEXT NOT NULL, orig TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (path, field));
CREATE TABLE IF NOT EXISTS runs (
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL, label TEXT, started INTEGER NOT NULL, finished INTEGER,
  status TEXT NOT NULL, files_ok INTEGER NOT NULL DEFAULT 0, files_conflict INTEGER NOT NULL DEFAULT 0,
  files_error INTEGER NOT NULL DEFAULT 0, summary TEXT NOT NULL DEFAULT '', undo_of INTEGER);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS journal (
  run_id INTEGER NOT NULL, path TEXT NOT NULL, field TEXT NOT NULL, before TEXT NOT NULL, after TEXT NOT NULL,
  ok INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (run_id, path, field));
"#;

pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn row_to_track(r: &rusqlite::Row) -> rusqlite::Result<TrackRow> {
    let fields: String = r.get(7)?;
    Ok(TrackRow {
        path: r.get(0)?,
        source_id: r.get(1)?,
        dir: r.get(2)?,
        file: r.get(3)?,
        ext: r.get(4)?,
        size: r.get::<_, i64>(5)? as u64,
        mtime: r.get(6)?,
        fields: serde_json::from_str(&fields).unwrap_or_default(),
        has_art: r.get::<_, i64>(8)? != 0,
        format: r.get(9)?,
        duration_ms: r.get::<_, Option<i64>>(10)?.map(|x| x as u64),
        riff_mismatch: r.get::<_, i64>(11)? != 0,
        error: r.get(12)?,
        readonly: r.get::<_, i64>(13)? != 0,
    })
}

const TRACK_COLS: &str = "path, source_id, dir, file, ext, size, mtime, fields, has_art, format, duration_ms, riff_mismatch, error, readonly";

impl Db {
    pub fn open(path: &Path) -> Result<Db> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn })
    }

    pub fn open_memory() -> Result<Db> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn })
    }

    // ---- sources

    pub fn add_source(&self, path: &str, name: &str) -> Result<Source> {
        self.conn.execute("INSERT INTO sources(path, name) VALUES (?1, ?2)", params![path, name])?;
        Ok(self.source(self.conn.last_insert_rowid())?.unwrap())
    }

    pub fn source(&self, id: i64) -> Result<Option<Source>> {
        Ok(self
            .conn
            .query_row("SELECT id, path, name, excludes, status, last_scan FROM sources WHERE id = ?1", [id], |r| {
                Ok(Source {
                    id: r.get(0)?,
                    path: r.get(1)?,
                    name: r.get(2)?,
                    excludes: serde_json::from_str(&r.get::<_, String>(3)?).unwrap_or_default(),
                    status: r.get(4)?,
                    last_scan: r.get(5)?,
                })
            })
            .optional()?)
    }

    pub fn sources(&self) -> Result<Vec<Source>> {
        let ids: Vec<i64> = self.conn.prepare("SELECT id FROM sources ORDER BY name")?.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
        Ok(ids.into_iter().filter_map(|id| self.source(id).ok().flatten()).collect())
    }

    pub fn set_source_status(&self, id: i64, status: &str, scanned: bool) -> Result<()> {
        if scanned {
            self.conn.execute("UPDATE sources SET status = ?2, last_scan = ?3 WHERE id = ?1", params![id, status, now()])?;
        } else {
            self.conn.execute("UPDATE sources SET status = ?2 WHERE id = ?1", params![id, status])?;
        }
        Ok(())
    }

    pub fn set_excludes(&self, id: i64, excludes: &[String]) -> Result<()> {
        self.conn.execute("UPDATE sources SET excludes = ?2 WHERE id = ?1", params![id, serde_json::to_string(excludes)?])?;
        Ok(())
    }

    pub fn remove_source(&self, id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM staged WHERE path IN (SELECT path FROM tracks WHERE source_id = ?1)", [id])?;
        tx.execute("DELETE FROM tracks WHERE source_id = ?1", [id])?;
        tx.execute("DELETE FROM sources WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }

    // ---- settings

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self.conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0)).optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute("INSERT OR REPLACE INTO settings(key, value) VALUES (?1, ?2)", params![key, value])?;
        Ok(())
    }

    // ---- tracks

    pub fn upsert_tracks(&self, rows: &[TrackRow]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut st = tx.prepare(&format!(
                "INSERT OR REPLACE INTO tracks({TRACK_COLS}) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)"
            ))?;
            for t in rows {
                st.execute(params![
                    t.path, t.source_id, t.dir, t.file, t.ext, t.size as i64, t.mtime, serde_json::to_string(&t.fields)?,
                    t.has_art as i64, t.format, t.duration_ms.map(|x| x as i64), t.riff_mismatch as i64, t.error, t.readonly as i64
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_tracks(&self, paths: &[String]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for p in paths {
            tx.execute("DELETE FROM tracks WHERE path = ?1", [p])?;
            tx.execute("DELETE FROM staged WHERE path = ?1", [p])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn tracks(&self) -> Result<Vec<TrackRow>> {
        let mut st = self.conn.prepare(&format!("SELECT {TRACK_COLS} FROM tracks ORDER BY dir, file"))?;
        let rows = st.query_map([], row_to_track)?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn track(&self, path: &str) -> Result<Option<TrackRow>> {
        Ok(self.conn.query_row(&format!("SELECT {TRACK_COLS} FROM tracks WHERE path = ?1"), [path], row_to_track).optional()?)
    }

    /// path -> (size, mtime) for files under `prefix` of a source, used by incremental scans.
    pub fn stamps(&self, source_id: i64, prefix: &str) -> Result<std::collections::HashMap<String, (u64, i64)>> {
        let mut st = self.conn.prepare("SELECT path, size, mtime FROM tracks WHERE source_id = ?1 AND substr(path, 1, length(?2)) = ?2")?;
        let m = st
            .query_map(params![source_id, prefix], |r| Ok((r.get::<_, String>(0)?, (r.get::<_, i64>(1)? as u64, r.get::<_, i64>(2)?))))?
            .collect::<Result<_, _>>()?;
        Ok(m)
    }

    // ---- staged changes (APP-STAGE-R1..R5)

    /// Set or clear staged values. A value equal to the indexed value removes the staged row (APP-STAGE-R2).
    pub fn stage_set(&self, changes: &[(String, String, String)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (path, field, value) in changes {
            let orig: Option<String> = tx
                .query_row("SELECT fields FROM tracks WHERE path = ?1", [path], |r| r.get::<_, String>(0))
                .optional()?
                .map(|f| serde_json::from_str::<Fields>(&f).unwrap_or_default().get(field).cloned().unwrap_or_default());
            let Some(orig) = orig else { continue };
            if &orig == value {
                tx.execute("DELETE FROM staged WHERE path = ?1 AND field = ?2", params![path, field])?;
            } else {
                tx.execute(
                    "INSERT INTO staged(path, field, orig, value) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(path, field) DO UPDATE SET value = excluded.value",
                    params![path, field, orig, value],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn staged(&self) -> Result<Vec<StagedChange>> {
        let mut st = self.conn.prepare("SELECT path, field, orig, value FROM staged ORDER BY path, field")?;
        let rows = st
            .query_map([], |r| Ok(StagedChange { path: r.get(0)?, field: r.get(1)?, orig: r.get(2)?, value: r.get(3)? }))?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    pub fn discard(&self, keys: &[(String, String)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (p, f) in keys {
            tx.execute("DELETE FROM staged WHERE path = ?1 AND field = ?2", params![p, f])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn discard_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM staged", [])?;
        Ok(())
    }

    // ---- runs and journal (APP-WRITE-R3, APP-UNDO)

    pub fn start_run(&self, kind: &str, label: Option<&str>, undo_of: Option<i64>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO runs(kind, label, started, status, undo_of) VALUES (?1, ?2, ?3, 'running', ?4)",
            params![kind, label, now(), undo_of],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn journal_before(&self, run: i64, path: &str, entries: &[(String, String, String)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (field, before, after) in entries {
            tx.execute(
                "INSERT OR REPLACE INTO journal(run_id, path, field, before, after, ok) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                params![run, path, field, before, after],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn journal_ok(&self, run: i64, path: &str) -> Result<()> {
        self.conn.execute("UPDATE journal SET ok = 1 WHERE run_id = ?1 AND path = ?2", params![run, path])?;
        Ok(())
    }

    pub fn journal(&self, run: i64) -> Result<Vec<(String, String, String, String)>> {
        let mut st = self.conn.prepare("SELECT path, field, before, after FROM journal WHERE run_id = ?1 AND ok = 1 ORDER BY path, field")?;
        let rows = st.query_map([run], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.collect::<Result<_, _>>()?;
        Ok(rows)
    }

    pub fn finish_run(&self, run: i64, status: &str, ok: i64, conflict: i64, error: i64, summary: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE runs SET finished = ?2, status = ?3, files_ok = ?4, files_conflict = ?5, files_error = ?6, summary = ?7 WHERE id = ?1",
            params![run, now(), status, ok, conflict, error, summary],
        )?;
        Ok(())
    }

    pub fn runs(&self) -> Result<Vec<RunInfo>> {
        let mut st = self.conn.prepare(
            "SELECT id, kind, label, started, finished, status, files_ok, files_conflict, files_error, summary, undo_of FROM runs ORDER BY id DESC LIMIT 200",
        )?;
        let rows = st
            .query_map([], |r| {
                Ok(RunInfo {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    label: r.get(2)?,
                    started: r.get(3)?,
                    finished: r.get(4)?,
                    status: r.get(5)?,
                    files_ok: r.get(6)?,
                    files_conflict: r.get(7)?,
                    files_error: r.get(8)?,
                    summary: r.get(9)?,
                    undo_of: r.get(10)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    /// APP-UNDO-R6: keep at most 200 runs and 90 days.
    pub fn prune_runs(&self) -> Result<()> {
        let cutoff = now() - 90 * 86_400;
        self.conn.execute(
            "DELETE FROM journal WHERE run_id IN (SELECT id FROM runs WHERE started < ?1 OR id NOT IN (SELECT id FROM runs ORDER BY id DESC LIMIT 200))",
            [cutoff],
        )?;
        self.conn.execute("DELETE FROM runs WHERE started < ?1 OR id NOT IN (SELECT id FROM runs ORDER BY id DESC LIMIT 200)", [cutoff])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(path: &str, title: &str) -> TrackRow {
        let mut f = Fields::new();
        f.insert("title".into(), title.into());
        TrackRow {
            path: path.into(), source_id: 1, dir: "/m".into(), file: path.rsplit('/').next().unwrap().into(), ext: "flac".into(),
            size: 10, mtime: 5, fields: f, has_art: false, format: "FLAC".into(), duration_ms: None, riff_mismatch: false, error: None, readonly: false,
        }
    }

    #[test]
    fn stage_keeps_orig_and_cancels_when_back_to_orig() {
        let db = Db::open_memory().unwrap();
        db.add_source("/m", "m").unwrap();
        db.upsert_tracks(&[row("/m/a.flac", "A")]).unwrap();
        db.stage_set(&[("/m/a.flac".into(), "title".into(), "B".into())]).unwrap();
        db.stage_set(&[("/m/a.flac".into(), "title".into(), "C".into())]).unwrap();
        let s = db.staged().unwrap();
        assert_eq!(s, vec![StagedChange { path: "/m/a.flac".into(), field: "title".into(), orig: "A".into(), value: "C".into() }]);
        db.stage_set(&[("/m/a.flac".into(), "title".into(), "A".into())]).unwrap();
        assert!(db.staged().unwrap().is_empty(), "APP-STAGE-R2");
    }

    #[test]
    fn staged_changes_survive_reopen() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("x.db");
        {
            let db = Db::open(&p).unwrap();
            db.add_source("/m", "m").unwrap();
            db.upsert_tracks(&[row("/m/a.flac", "A")]).unwrap();
            db.stage_set(&[("/m/a.flac".into(), "title".into(), "B".into())]).unwrap();
        } // dropped without any explicit save
        let db = Db::open(&p).unwrap();
        assert_eq!(db.staged().unwrap().len(), 1, "APP-STAGE-R5");
    }

    #[test]
    fn stamps_by_prefix() {
        let db = Db::open_memory().unwrap();
        db.add_source("/m", "m").unwrap();
        db.upsert_tracks(&[row("/m/x/a.flac", "A"), row("/m/y/b.flac", "B")]).unwrap();
        assert_eq!(db.stamps(1, "/m/x/").unwrap().len(), 1);
        assert_eq!(db.stamps(1, "/m/").unwrap().len(), 2);
    }
}
