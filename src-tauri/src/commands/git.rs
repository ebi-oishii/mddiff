use std::path::PathBuf;

use mdv_core::diff::{DiffLine, HunkSummary};
use mdv_core::git::{BaseOption, DEFAULT_BASE};

fn resolve(base: Option<String>) -> String {
    base.filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_BASE.to_string())
}

#[tauri::command]
pub async fn git_is_repo(path: PathBuf) -> Result<bool, String> {
    Ok(mdv_core::git::is_in_repo(&path))
}

#[tauri::command]
pub async fn git_list_bases(
    path: PathBuf,
    current_text: Option<String>,
) -> Result<Vec<BaseOption>, String> {
    mdv_core::git::list_bases(&path, current_text.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_hunks(
    path: PathBuf,
    current_text: String,
    base: Option<String>,
) -> Result<Vec<HunkSummary>, String> {
    let base = resolve(base);
    mdv_core::git::diff_text_against_base(&path, &current_text, &base).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_full_diff(
    path: PathBuf,
    current_text: String,
    base: Option<String>,
) -> Result<Vec<DiffLine>, String> {
    let base = resolve(base);
    mdv_core::git::full_diff_against_base(&path, &current_text, &base).map_err(|e| e.to_string())
}
