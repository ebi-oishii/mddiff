use mdv_core::diff::{line_diff, full_diff, DiffLine, HunkSummary};
use mdv_core::git::SideBySidePayload;

/// Diff two text blobs directly, no Git involvement. Used for the "Compare
/// with disk" path triggered from the external-change banner.

#[tauri::command]
pub async fn diff_text_hunks(
    old_text: String,
    new_text: String,
) -> Result<Vec<HunkSummary>, String> {
    Ok(line_diff(&old_text, &new_text))
}

#[tauri::command]
pub async fn diff_text_full(
    old_text: String,
    new_text: String,
) -> Result<Vec<DiffLine>, String> {
    Ok(full_diff(&old_text, &new_text))
}

#[tauri::command]
pub async fn diff_text_side_by_side(
    old_text: String,
    new_text: String,
) -> Result<SideBySidePayload, String> {
    let hunks = line_diff(&old_text, &new_text);
    Ok(SideBySidePayload {
        old_text,
        new_text,
        hunks,
    })
}
