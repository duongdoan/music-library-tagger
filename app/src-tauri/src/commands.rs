//! Tauri commands: the only surface the UI talks to.
use crate::apply::{self, ApplyItem, FileResult, Report, UndoPlan, RIFF_SYNC};
use crate::db::Db;
use crate::model::{RunInfo, Source, StageInput, StagedChange, TrackRow, FIELDS};
use crate::norm::{normalize, validate};
use crate::scan::{self, Progress, Summary};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

pub struct AppState {
    pub db: Arc<Mutex<Db>>,
    pub scan_cancel: Arc<AtomicBool>,
    pub apply_cancel: Arc<AtomicBool>,
}

type CmdResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[tauri::command]
pub fn list_sources(state: State<AppState>) -> CmdResult<Vec<Source>> {
    state.db.lock().unwrap().sources().map_err(err)
}

/// APP-LIB-OPEN validation messages are the spec's verbatim strings.
#[tauri::command]
pub fn add_source(state: State<AppState>, path: String) -> CmdResult<Source> {
    let db = state.db.lock().unwrap();
    let p = path.trim_end_matches('/').to_string();
    for s in db.sources().map_err(err)? {
        if s.path == p {
            return Err("Thư mục này đã có trong thư viện".into());
        }
        if p.starts_with(&format!("{}/", s.path)) {
            return Err(format!("Thư mục này đã nằm trong nguồn «{}»", s.name));
        }
    }
    let name = Path::new(&p).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.clone());
    let src = db.add_source(&p, &name).map_err(err)?;
    // APP-SCAN-R10: suggest excluding music-player backups
    db.set_excludes(src.id, &["roon-backup".to_string()]).map_err(err)?;
    db.source(src.id).map_err(err)?.ok_or_else(|| "nguồn không tồn tại".into())
}

#[tauri::command]
pub fn remove_source(state: State<AppState>, id: i64) -> CmdResult<()> {
    state.db.lock().unwrap().remove_source(id).map_err(err)
}

#[tauri::command]
pub fn set_excludes(state: State<AppState>, id: i64, excludes: Vec<String>) -> CmdResult<()> {
    state.db.lock().unwrap().set_excludes(id, &excludes).map_err(err)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchEvent {
    rows: Vec<TrackRow>,
    removed: Vec<String>,
}

/// Scan a source, or only `folder` inside it (APP-SCAN-R12). Progress and rows stream as events.
#[tauri::command]
pub async fn scan_source(app: AppHandle, state: State<'_, AppState>, id: i64, folder: Option<String>) -> CmdResult<Summary> {
    let db = state.db.clone();
    let cancel = state.scan_cancel.clone();
    cancel.store(false, Ordering::Relaxed);
    tauri::async_runtime::spawn_blocking(move || {
        let src = db.lock().unwrap().source(id).map_err(err)?.ok_or("nguồn không tồn tại")?;
        let root = PathBuf::from(folder.unwrap_or_else(|| src.path.clone()));
        let a1 = app.clone();
        let a2 = app.clone();
        scan::scan(
            &db,
            &src,
            &root,
            &cancel,
            16,
            &move |p: Progress| {
                let _ = a1.emit("scan-progress", p);
            },
            &move |rows, removed| {
                let _ = a2.emit("scan-batch", BatchEvent { rows, removed });
            },
        )
        .map_err(err)
    })
    .await
    .map_err(err)?
}

#[tauri::command]
pub fn cancel_scan(state: State<AppState>) {
    state.scan_cancel.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn load_tracks(state: State<'_, AppState>) -> CmdResult<Vec<TrackRow>> {
    let db = state.db.clone();
    tauri::async_runtime::spawn_blocking(move || db.lock().unwrap().tracks().map_err(err)).await.map_err(err)?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageResult {
    pub applied: usize,
    pub rejected: Vec<String>,
}

/// Values are normalised (APP-TAG-R4) and validated (APP-TAG-R6) here as well as in the UI.
#[tauri::command]
pub fn stage_set(state: State<AppState>, changes: Vec<StageInput>) -> CmdResult<StageResult> {
    let mut ok = vec![];
    let mut rejected = vec![];
    for c in changes {
        if c.field == RIFF_SYNC {
            ok.push((c.path, c.field, c.value));
            continue;
        }
        if !FIELDS.contains(&c.field.as_str()) {
            rejected.push(format!("{}: trường không hỗ trợ", c.field));
            continue;
        }
        let v = normalize(&c.value);
        if let Some(m) = validate(&c.field, &v) {
            rejected.push(format!("{}: {m}", c.path));
            continue;
        }
        ok.push((c.path, c.field, v));
    }
    let n = ok.len();
    state.db.lock().unwrap().stage_set(&ok).map_err(err)?;
    Ok(StageResult { applied: n, rejected })
}

#[tauri::command]
pub fn stage_list(state: State<AppState>) -> CmdResult<Vec<StagedChange>> {
    state.db.lock().unwrap().staged().map_err(err)
}

#[tauri::command]
pub fn stage_discard(state: State<AppState>, keys: Vec<ApplyItem>) -> CmdResult<()> {
    let k: Vec<(String, String)> = keys.into_iter().map(|i| (i.path, i.field)).collect();
    state.db.lock().unwrap().discard(&k).map_err(err)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplyEvent {
    done: usize,
    total: usize,
    result: FileResult,
}

#[tauri::command]
pub async fn apply_run(app: AppHandle, state: State<'_, AppState>, items: Vec<ApplyItem>, undo_of: Option<i64>) -> CmdResult<Report> {
    let db = state.db.clone();
    let cancel = state.apply_cancel.clone();
    cancel.store(false, Ordering::Relaxed);
    tauri::async_runtime::spawn_blocking(move || {
        let in_place = db.lock().unwrap().setting("write_mode").ok().flatten().as_deref() == Some("in_place");
        apply::apply(&db, &items, undo_of, in_place, &cancel, &move |done, total, r| {
            let _ = app.emit("apply-progress", ApplyEvent { done, total, result: r.clone() });
        })
        .map_err(err)
    })
    .await
    .map_err(err)?
}

#[tauri::command]
pub fn cancel_apply(state: State<AppState>) {
    state.apply_cancel.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn list_runs(state: State<AppState>) -> CmdResult<Vec<RunInfo>> {
    state.db.lock().unwrap().runs().map_err(err)
}

#[tauri::command]
pub fn undo_plan(state: State<AppState>, run_id: i64) -> CmdResult<UndoPlan> {
    apply::undo_plan(&state.db.lock().unwrap(), run_id).map_err(err)
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    state.db.lock().unwrap().setting(&key).map_err(err)
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    state.db.lock().unwrap().set_setting(&key, &value).map_err(err)
}
