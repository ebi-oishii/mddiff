use std::path::PathBuf;

use mddiff_core::diff::{DiffLine, HunkSummary};
use mddiff_core::git::{BaseOption, SideBySidePayload, DEFAULT_BASE};

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
