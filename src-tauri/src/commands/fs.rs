use std::path::PathBuf;

use base64::Engine;

#[tauri::command]
pub async fn read_text_file(path: PathBuf, force: Option<bool>) -> Result<String, String> {
    mdv_core::fs::read_text_file_with(&path, force.unwrap_or(false)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn file_size(path: PathBuf) -> Result<u64, String> {
    mdv_core::fs::file_size(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_text_file(
    path: PathBuf,
    content: String,
    #[cfg(not(any(target_os = "android", target_os = "ios")))] watcher: tauri::State<
        '_,
        crate::commands::watcher::WatcherState,
    >,
) -> Result<(), String> {
    // Suppress the file watcher for the next 500ms so our own write doesn't
    // bounce back as an "external change".
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    crate::commands::watcher::mark_self_write(&watcher, &path);
    mdv_core::fs::write_text_file(&path, &content).map_err(|e| e.to_string())
}

/// Binary write path. Used for DOCX export (and any future binary format).
/// JS encodes the bytes as standard base64 so the IPC stays a normal JSON
/// string instead of streaming a `Vec<u8>`.
#[tauri::command]
pub async fn write_binary_file(path: PathBuf, base64: String) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64.as_bytes())
        .map_err(|e| format!("invalid base64: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}
