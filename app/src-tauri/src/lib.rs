pub mod apply;
pub mod commands;
pub mod db;
pub mod model;
pub mod norm;
pub mod scan;
pub mod tags;

use commands::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = db::Db::open(&dir.join("library.db"))?;
            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                scan_cancel: Arc::new(AtomicBool::new(false)),
                apply_cancel: Arc::new(AtomicBool::new(false)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_sources,
            commands::add_source,
            commands::remove_source,
            commands::set_excludes,
            commands::scan_source,
            commands::cancel_scan,
            commands::load_tracks,
            commands::stage_set,
            commands::stage_list,
            commands::stage_discard,
            commands::apply_run,
            commands::cancel_apply,
            commands::list_runs,
            commands::undo_plan,
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
