mod commands;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod menu;

use std::path::PathBuf;
use std::sync::Mutex;

use commands::fs::{file_size, read_text_file, write_binary_file, write_text_file};

/// File path queued from the OS (CLI argv at launch, or "Open With" from
/// Finder on macOS). The frontend drains this on mount via `pending_file_open`
/// and loads the file. While the app is running, OS-level file-open events
/// (macOS `RunEvent::Opened`) instead push the path here AND emit a
/// `file-open` event so an already-mounted frontend can swap to the new file
/// without a window reload.
#[derive(Default)]
struct PendingFile(Mutex<Option<PathBuf>>);

/// Walk argv looking for the first non-flag arg that resolves to an existing
/// file. Robust to mddiff being launched with flags we don't recognize, so
/// extending the CLI later doesn't break this.
fn parse_argv_file() -> Option<PathBuf> {
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        if arg.starts_with('-') {
            continue;
        }
        let p = PathBuf::from(&arg);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Returns the OS-queued file path (if any) and clears the queue. Frontend
/// calls this once on mount.
#[tauri::command]
fn pending_file_open(state: tauri::State<'_, PendingFile>) -> Option<String> {
    state
        .0
        .lock()
        .ok()
        .and_then(|mut g| g.take().map(|p| p.to_string_lossy().into_owned()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(PendingFile::default());

    // The file watcher needs shared state across commands and the watcher
    // callback, so it's owned by Tauri's State container.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.manage(commands::watcher::WatcherState::default());

    // Native menu bar on desktop. Each item emits a "menu-event" payload
    // (its string id) that the frontend dispatches in +page.svelte.
    //
    // Also seed `PendingFile` from argv so OS double-click / "Open With" on
    // Windows / Linux (which launches the app with the file path in argv)
    // reaches the frontend on first mount.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.setup(|app| {
        use tauri::Emitter;
        use tauri::Manager;
        let menu_bar = menu::build(&app.handle().clone())?;
        app.set_menu(menu_bar)?;
        app.on_menu_event(|app, event| {
            let _ = app.emit("menu-event", event.id().as_ref());
        });

        if let Some(p) = parse_argv_file() {
            if let Some(state) = app.try_state::<PendingFile>() {
                if let Ok(mut g) = state.0.lock() {
                    *g = Some(p);
                }
            }
        }
        Ok(())
    });

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        read_text_file,
        write_text_file,
        write_binary_file,
        file_size,
        pending_file_open,
        commands::git::git_is_repo,
        commands::git::git_list_bases,
        commands::git::git_hunks,
        commands::git::git_full_diff,
        commands::git::git_side_by_side,
        commands::git::git_read_at,
        commands::git::git_blame,
        commands::history::snapshot_save,
        commands::history::snapshot_list,
        commands::history::snapshot_read,
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
    let builder = builder
        .invoke_handler(tauri::generate_handler![read_text_file, write_text_file, file_size]);

    // `.build(...).run(callback)` instead of `.run(...)` so we can listen for
    // macOS `RunEvent::Opened` — that's where Finder's "Open With" events
    // arrive (argv on macOS doesn't carry them).
    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // `RunEvent::Opened` is macOS-only — Tauri doesn't expose the variant
        // on Windows / Linux, so the whole handler is gated. On those
        // platforms file-open is handled via argv at startup; runtime
        // "Open With" forwarding would need single-instance plugin (Phase 2).
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = event {
            use tauri::Emitter;
            use tauri::Manager;
            for url in urls {
                if let Ok(path) = url.to_file_path() {
                    let path_str = path.to_string_lossy().into_owned();
                    // Update queue for cold-mount consumers (in case the
                    // event arrives before the frontend has subscribed).
                    if let Some(state) = app_handle.try_state::<PendingFile>() {
                        if let Ok(mut g) = state.0.lock() {
                            *g = Some(path);
                        }
                    }
                    // And emit for already-running frontends.
                    let _ = app_handle.emit("file-open", path_str);
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Suppress unused-variable warnings on non-mac.
            let _ = (app_handle, event);
        }
    });
}
