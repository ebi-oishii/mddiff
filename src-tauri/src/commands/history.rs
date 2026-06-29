//! Tauri command bridge for `mddiff_core::history`. Resolves the per-app
//! data dir from `tauri::Manager::path()`, then forwards calls.

use std::path::PathBuf;

use mddiff_core::history::SnapshotMeta;
use tauri::{AppHandle, Manager, Runtime};

/// How many snapshots we keep per file. Markdown files are small so the
/// per-file cap matters more than total bytes. 100 is enough for several
/// days of heavy editing.
const KEEP_PER_FILE: usize = 100;

fn data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn snapshot_save<R: Runtime>(
    app: AppHandle<R>,
    path: PathBuf,
    content: String,
) -> Result<Option<SnapshotMeta>, String> {
    let dir = data_dir(&app)?;
    let meta = mddiff_core::history::snapshot_save(&dir, &path, &content)
        .map_err(|e| e.to_string())?;
    // Fire-and-forget prune so the per-file dir doesn't grow unbounded.
    let _ = mddiff_core::history::prune(&dir, &path, KEEP_PER_FILE);
    Ok(meta)
}

#[tauri::command]
pub async fn snapshot_list<R: Runtime>(
    app: AppHandle<R>,
    path: PathBuf,
) -> Result<Vec<SnapshotMeta>, String> {
    let dir = data_dir(&app)?;
    mddiff_core::history::list(&dir, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn snapshot_read<R: Runtime>(
    app: AppHandle<R>,
    path: PathBuf,
    id: String,
) -> Result<String, String> {
    let dir = data_dir(&app)?;
    mddiff_core::history::read(&dir, &path, &id).map_err(|e| e.to_string())
}
