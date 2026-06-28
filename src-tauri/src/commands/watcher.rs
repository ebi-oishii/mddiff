use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

/// Held in `tauri::State`. Watches the currently-open file and emits
/// `file-external-change` events when something changes on disk.
///
/// Self-write suppression: `write_text_file` marks an Instant 500ms in the
/// future before writing. Events arriving before that deadline are ignored so
/// our own save doesn't trip the watcher.
pub struct WatcherState {
    debouncer: Mutex<Option<Debouncer<RecommendedWatcher>>>,
    current_path: Mutex<Option<PathBuf>>,
    self_write_until: Arc<Mutex<Option<Instant>>>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            debouncer: Mutex::new(None),
            current_path: Mutex::new(None),
            self_write_until: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
enum ChangeKind {
    Modified,
    Removed,
}

#[derive(Serialize, Clone)]
struct ChangePayload {
    path: String,
    kind: ChangeKind,
}

/// Called from `write_text_file` right before the actual write. Sets a 500ms
/// suppression window so the watcher won't report our own save back to us.
pub fn mark_self_write(state: &WatcherState, _path: &Path) {
    if let Ok(mut s) = state.self_write_until.lock() {
        *s = Some(Instant::now() + Duration::from_millis(500));
    }
}

#[tauri::command]
pub fn start_watch(
    path: String,
    state: State<'_, WatcherState>,
    app: AppHandle,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    // Watch the parent directory rather than the file itself. FSEvents on
    // macOS can drop events for editors that swap files atomically (write
    // tmp + rename); directory-level watch survives that pattern.
    let parent = path_buf
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "file has no parent directory".to_string())?;

    let target_path = path_buf.clone();
    let app_handle = app.clone();
    let self_write_until = state.self_write_until.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(150),
        move |res: DebounceEventResult| {
            let events = match res {
                Ok(events) => events,
                Err(_) => return,
            };

            // Self-write suppression. One-shot: clear the deadline once we've
            // honored it so the next external change always fires.
            {
                if let Ok(mut lock) = self_write_until.lock() {
                    if let Some(deadline) = *lock {
                        if Instant::now() < deadline {
                            return;
                        }
                        *lock = None;
                    }
                }
            }

            // We watch the parent dir so we'll see events for siblings too.
            // Only emit when the target file itself is touched.
            let matched = events.iter().any(|e| e.path == target_path);
            if !matched {
                return;
            }

            let kind = if target_path.exists() {
                ChangeKind::Modified
            } else {
                ChangeKind::Removed
            };

            let _ = app_handle.emit(
                "file-external-change",
                ChangePayload {
                    path: target_path.to_string_lossy().to_string(),
                    kind,
                },
            );
        },
    )
    .map_err(|e| e.to_string())?;

    debouncer
        .watcher()
        .watch(&parent, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    // Replacing any previous watcher drops it here, which is what we want
    // when the user opens a different file.
    *state.debouncer.lock().map_err(|e| e.to_string())? = Some(debouncer);
    *state.current_path.lock().map_err(|e| e.to_string())? = Some(path_buf);

    Ok(())
}

#[tauri::command]
pub fn stop_watch(state: State<'_, WatcherState>) -> Result<(), String> {
    *state.debouncer.lock().map_err(|e| e.to_string())? = None;
    *state.current_path.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}
