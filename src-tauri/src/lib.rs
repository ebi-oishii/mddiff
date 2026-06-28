mod commands;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod menu;

use commands::fs::{file_size, read_text_file, write_binary_file, write_text_file};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init());

    // The file watcher needs shared state across commands and the watcher
    // callback, so it's owned by Tauri's State container.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.manage(commands::watcher::WatcherState::default());

    // Native menu bar on desktop. Each item emits a "menu-event" payload
    // (its string id) that the frontend dispatches in +page.svelte.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.setup(|app| {
        use tauri::Emitter;
        let menu_bar = menu::build(&app.handle().clone())?;
        app.set_menu(menu_bar)?;
        app.on_menu_event(|app, event| {
            let _ = app.emit("menu-event", event.id().as_ref());
        });
        Ok(())
    });

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        read_text_file,
        write_text_file,
        write_binary_file,
        file_size,
        commands::git::git_is_repo,
        commands::git::git_list_bases,
        commands::git::git_hunks,
        commands::git::git_full_diff,
        commands::git::git_side_by_side,
        commands::git::git_read_at,
        commands::mddiff::mddiff_pack,
        commands::mddiff::mddiff_extract_body,
        commands::watcher::start_watch,
        commands::watcher::stop_watch,
        commands::diff::diff_text_hunks,
        commands::diff::diff_text_full,
        commands::diff::diff_text_side_by_side,
        commands::clipboard::paste_image,
    ]);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let builder = builder.invoke_handler(tauri::generate_handler![read_text_file, write_text_file, file_size]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
