use std::path::PathBuf;

use base64::Engine;

#[tauri::command]
pub async fn read_text_file(path: PathBuf) -> Result<String, String> {
    mdv_core::fs::read_text_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_text_file(path: PathBuf, content: String) -> Result<(), String> {
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
