use std::path::Path;

use thiserror::Error;

/// Refuse to open files larger than this. CodeMirror, markdown-it and Milkdown
/// all degrade sharply past a few MB; loading multi-MB documents pegs the UI
/// and risks OOM on mobile.
pub const MAX_OPEN_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("file is {actual} bytes, exceeds the {limit}-byte limit")]
    TooLarge { actual: u64, limit: u64 },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_text_file(path: &Path) -> Result<String, ReadError> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_OPEN_BYTES {
        return Err(ReadError::TooLarge {
            actual: meta.len(),
            limit: MAX_OPEN_BYTES,
        });
    }
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_text_file(path: &Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}
