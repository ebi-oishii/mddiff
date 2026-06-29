use std::path::PathBuf;

use mddiff_core::blame::BlameLine;
use mddiff_core::diff::{DiffLine, HunkSummary};
use mddiff_core::git::{BaseOption, SideBySidePayload, DEFAULT_BASE};
use tauri::{AppHandle, Manager, Runtime};

fn resolve(base: Option<String>) -> String {
    base.filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_BASE.to_string())
}

#[tauri::command]
pub async fn git_is_repo(path: PathBuf) -> Result<bool, String> {
    Ok(mddiff_core::git::is_in_repo(&path))
}

#[tauri::command]
pub async fn git_list_bases(
    path: PathBuf,
    current_text: Option<String>,
) -> Result<Vec<BaseOption>, String> {
    mddiff_core::git::list_bases(&path, current_text.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_hunks(
    path: PathBuf,
    current_text: String,
    base: Option<String>,
) -> Result<Vec<HunkSummary>, String> {
    let base = resolve(base);
    mddiff_core::git::diff_text_against_base(&path, &current_text, &base).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_full_diff(
    path: PathBuf,
    current_text: String,
    base: Option<String>,
) -> Result<Vec<DiffLine>, String> {
    let base = resolve(base);
    mddiff_core::git::full_diff_against_base(&path, &current_text, &base).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_side_by_side(
    path: PathBuf,
    current_text: String,
    base: Option<String>,
) -> Result<SideBySidePayload, String> {
    let base = resolve(base);
    mddiff_core::git::side_by_side_against_base(&path, &current_text, &base)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_read_at(path: PathBuf, revspec: String) -> Result<String, String> {
    mddiff_core::git::read_at(&path, &revspec).map_err(|e| e.to_string())
}

/// Per-line blame across Git + local save snapshots. The snapshot timestamp
/// is sourced from the app data dir (see commands::history) so the blame
/// view can attribute non-HEAD lines to a "local · <date>" entry.
///
/// Wrapped in `catch_unwind`: blame walks libgit2 internals and a panic
/// there would otherwise tear down the whole webview process. The frontend
/// shows the error string instead.
#[tauri::command]
pub async fn git_blame<R: Runtime>(
    app: AppHandle<R>,
    path: PathBuf,
    current_text: String,
) -> Result<Vec<BlameLine>, String> {
    let latest_ts_ms = app
        .path()
        .app_data_dir()
        .ok()
        .and_then(|dir| mddiff_core::history::list(&dir, &path).ok())
        .and_then(|snaps| snaps.first().map(|m| m.timestamp_ms));
    let path_clone = path.clone();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mddiff_core::blame::compute(&path_clone, &current_text, latest_ts_ms)
    }))
    .map_err(|p| {
        // Best-effort message extraction; panic payloads are most often &str / String.
        let msg = p
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "blame panicked".to_string());
        format!("blame computation panicked: {msg}")
    })?
    .map_err(|e| e.to_string())
}
