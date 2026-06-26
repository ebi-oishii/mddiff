use std::path::PathBuf;

#[tauri::command]
pub async fn read_text_file(path: PathBuf) -> Result<String, String> {
    mdv_core::fs::read_text_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_text_file(path: PathBuf, content: String) -> Result<(), String> {
    mdv_core::fs::write_text_file(&path, &content).map_err(|e| e.to_string())
}
