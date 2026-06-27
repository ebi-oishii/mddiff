mod commands;

use commands::fs::{read_text_file, write_binary_file, write_text_file};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        read_text_file,
        write_text_file,
        write_binary_file,
        commands::git::git_is_repo,
        commands::git::git_list_bases,
        commands::git::git_hunks,
        commands::git::git_full_diff,
        commands::git::git_side_by_side,
    ]);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let builder = builder.invoke_handler(tauri::generate_handler![read_text_file, write_text_file]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
