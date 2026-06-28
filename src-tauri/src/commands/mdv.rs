use std::path::PathBuf;

use serde::Serialize;

#[derive(Serialize)]
pub struct PackResponse {
    pub content: String,
    pub commit_count: usize,
    pub snapshot_count: usize,
    pub bundle_bytes: usize,
}

#[tauri::command]
pub async fn mdv_pack(
    path: PathBuf,
    current_text: String,
    base: String,
) -> Result<PackResponse, String> {
    let (author_name, author_email) = mdv_core::git::user_identity(&path);
    let result = mdv_core::pack::pack(
        &path,
        &current_text,
        &base,
        &author_name,
        author_email.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    Ok(PackResponse {
        content: result.content,
        commit_count: result.commit_count,
        snapshot_count: result.snapshot_count,
        bundle_bytes: result.bundle_bytes,
    })
}

#[tauri::command]
pub fn mdv_extract_body(content: String) -> String {
    mdv_core::pack::extract_body(&content)
}
