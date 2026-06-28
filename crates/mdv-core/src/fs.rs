use std::path::Path;

use thiserror::Error;

/// Soft warning threshold. CodeMirror, markdown-it and Milkdown all degrade
/// sharply past a few MB; reading multi-MB documents pegs the UI and risks
/// OOM on mobile. Callers should ask the user before opening at or above
/// this size (via `force=true`).
pub const MAX_OPEN_BYTES: u64 = 5 * 1024 * 1024;

/// Absolute ceiling. We refuse to open above this even with `force=true` —
/// at 100MB the WebView frequently freezes for tens of seconds or OOM-kills.
pub const HARD_CAP_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ReadError {
    /// File exceeds the soft threshold but is below the hard cap. The caller
    /// can retry with `force=true` after confirming with the user.
    #[error("file is {actual} bytes, exceeds the {limit}-byte warning threshold")]
    TooLarge { actual: u64, limit: u64 },
    /// File exceeds the hard cap. Even `force=true` cannot open it.
    #[error("file is {actual} bytes, exceeds the {limit}-byte hard limit — refusing to open")]
    HardCap { actual: u64, limit: u64 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_text_file(path: &Path) -> Result<String, ReadError> {
    read_text_file_with(path, false)
}

/// `force=true` skips the `MAX_OPEN_BYTES` check (used after the user
/// confirms via the warning dialog or by passing `--force` on the TUI).
/// The `HARD_CAP_BYTES` ceiling is always enforced.
pub fn read_text_file_with(path: &Path, force: bool) -> Result<String, ReadError> {
    let meta = std::fs::metadata(path)?;
    let size = meta.len();
    if size > HARD_CAP_BYTES {
        return Err(ReadError::HardCap {
            actual: size,
            limit: HARD_CAP_BYTES,
        });
    }
    if size > MAX_OPEN_BYTES && !force {
        return Err(ReadError::TooLarge {
            actual: size,
            limit: MAX_OPEN_BYTES,
        });
    }
    Ok(std::fs::read_to_string(path)?)
}

/// File size in bytes without reading the contents. Used by the GUI to
/// decide whether to show the large-file warning before the read.
pub fn file_size(path: &Path) -> std::io::Result<u64> {
    Ok(std::fs::metadata(path)?.len())
}

pub fn write_text_file(path: &Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}
