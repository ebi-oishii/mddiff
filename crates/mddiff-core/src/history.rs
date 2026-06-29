//! Save-event snapshot history.
//!
//! Each save can drop a zstd-compressed copy of the buffer into the user's
//! per-app data dir, keyed by a sha256 of the file's absolute path. Snapshots
//! are local-only: never pushed, never shared across machines, never
//! intersected with Git history at write time.
//!
//! Layout under `<data_dir>/`:
//!
//! ```text
//! mddiff/
//!   snapshots/
//!     <path-hash>/
//!       meta.json           # one file: {"original_path": "..."} for debug
//!       <id>.zst            # body content, zstd-compressed
//! ```
//!
//! `<id>` is `<unix_ms>-<sha8>`: epoch ms for ordering + a short hash of the
//! content for fast equality check (the prune path uses it to skip writing
//! a snapshot when the latest entry already has the same content).
//!
//! Caller (the Tauri command layer) owns:
//! - Deciding where the data dir lives (`tauri::Manager::path().app_data_dir`)
//! - Calling `snapshot_save` only on successful disk writes
//! - Calling `prune` to keep the per-file snapshot count bounded
//!
//! This module knows nothing about Tauri — it's pure filesystem + zstd.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// One on-disk snapshot. The body is read on demand via `read`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// `<unix_ms>-<sha8>` — opaque to callers; pass back into `read`.
    pub id: String,
    /// Milliseconds since Unix epoch — sort key, also what the UI labels.
    pub timestamp_ms: i64,
    /// Compressed bytes on disk. Useful for the UI to flag huge accumulations.
    pub size_bytes: u64,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("snapshot not found: {0}")]
    NotFound(String),
    #[error("snapshot blob is not valid utf-8")]
    NotUtf8,
}

/// Build the per-file snapshot directory under `<data_dir>/mddiff/snapshots/`.
/// Path is hashed so weird characters / very long absolute paths don't end up
/// in the FS layout. The directory is created lazily by `snapshot_save`.
pub fn snapshot_dir(data_dir: &Path, file_abs: &Path) -> PathBuf {
    let hash = sha256_hex(file_abs.to_string_lossy().as_bytes());
    let mut dir = data_dir.to_path_buf();
    dir.push("mddiff");
    dir.push("snapshots");
    dir.push(&hash[..16]);
    dir
}

/// Persist `content` to a new snapshot under `<data_dir>/mddiff/snapshots/...`.
///
/// Returns `Ok(None)` (without writing) when the most recent snapshot has
/// identical content — saving the same buffer twice in a row shouldn't burn
/// a slot. Returns `Ok(Some(meta))` for a fresh snapshot.
pub fn snapshot_save(
    data_dir: &Path,
    file_abs: &Path,
    content: &str,
) -> Result<Option<SnapshotMeta>, HistoryError> {
    let dir = snapshot_dir(data_dir, file_abs);
    fs::create_dir_all(&dir)?;

    // Dedup against the latest snapshot — the content hash is encoded into
    // the id, so a directory listing is enough; we don't need to read bodies.
    let content_sha8 = &sha256_hex(content.as_bytes())[..8];
    if let Some(latest) = list(data_dir, file_abs)?.into_iter().next() {
        if latest.id.ends_with(content_sha8) {
            return Ok(None);
        }
    }

    let ts = now_ms();
    let id = format!("{ts}-{content_sha8}");
    let path = dir.join(format!("{id}.zst"));
    let compressed = zstd::encode_all(content.as_bytes(), 19)?;
    fs::write(&path, &compressed)?;

    // Best-effort meta sidecar — purely diagnostic; if write fails (eg.
    // read-only FS race) we don't surface that to the caller.
    let _ = write_meta(&dir, file_abs);

    let size = compressed.len() as u64;
    Ok(Some(SnapshotMeta {
        id,
        timestamp_ms: ts,
        size_bytes: size,
    }))
}

/// Newest first. Returns an empty vec when the dir doesn't exist yet.
pub fn list(data_dir: &Path, file_abs: &Path) -> Result<Vec<SnapshotMeta>, HistoryError> {
    let dir = snapshot_dir(data_dir, file_abs);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // <ts>-<sha8>.zst
        if let Some(id) = name.strip_suffix(".zst") {
            // Parse the ts prefix; bad names are skipped silently — they were
            // probably written by a different tool / partial copy.
            if let Some((ts_str, _)) = id.split_once('-') {
                if let Ok(ts) = ts_str.parse::<i64>() {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push(SnapshotMeta {
                        id: id.to_string(),
                        timestamp_ms: ts,
                        size_bytes: size,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    Ok(out)
}

/// Read a snapshot's body. `id` is what `list` returns (no `.zst` suffix).
pub fn read(data_dir: &Path, file_abs: &Path, id: &str) -> Result<String, HistoryError> {
    let dir = snapshot_dir(data_dir, file_abs);
    let path = dir.join(format!("{id}.zst"));
    if !path.exists() {
        return Err(HistoryError::NotFound(id.to_string()));
    }
    let mut decoder = zstd::stream::read::Decoder::new(fs::File::open(&path)?)?;
    let mut s = String::new();
    decoder.read_to_string(&mut s).map_err(|_| HistoryError::NotUtf8)?;
    Ok(s)
}

/// Keep only the `keep` newest snapshots. Quietly skips errors on individual
/// deletes — pruning is best-effort housekeeping, not a correctness step.
pub fn prune(data_dir: &Path, file_abs: &Path, keep: usize) -> Result<(), HistoryError> {
    let all = list(data_dir, file_abs)?;
    if all.len() <= keep {
        return Ok(());
    }
    let dir = snapshot_dir(data_dir, file_abs);
    for meta in &all[keep..] {
        let _ = fs::remove_file(dir.join(format!("{}.zst", meta.id)));
    }
    Ok(())
}

fn write_meta(dir: &Path, file_abs: &Path) -> Result<(), HistoryError> {
    let meta = serde_json::json!({
        "original_path": file_abs.to_string_lossy(),
        "schema": 1,
    });
    fs::write(dir.join("meta.json"), meta.to_string())?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    let mut s = String::with_capacity(64);
    for b in digest.iter() {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("mddiff-history-test-{name}-{}", now_ms()));
        p
    }

    #[test]
    fn save_then_read_roundtrips() {
        let data_dir = tmp_dir("roundtrip");
        let file = PathBuf::from("/tmp/sample.md");
        let body = "# Hello\n\nWorld\n";
        let meta = snapshot_save(&data_dir, &file, body).unwrap().unwrap();
        let got = read(&data_dir, &file, &meta.id).unwrap();
        assert_eq!(got, body);
    }

    #[test]
    fn identical_save_returns_none() {
        let data_dir = tmp_dir("dedup");
        let file = PathBuf::from("/tmp/sample.md");
        let body = "same content\n";
        snapshot_save(&data_dir, &file, body).unwrap().unwrap();
        let again = snapshot_save(&data_dir, &file, body).unwrap();
        assert!(again.is_none());
    }

    #[test]
    fn list_orders_newest_first() {
        let data_dir = tmp_dir("order");
        let file = PathBuf::from("/tmp/sample.md");
        snapshot_save(&data_dir, &file, "v1").unwrap();
        // Spin briefly to advance the ms timestamp deterministically.
        std::thread::sleep(std::time::Duration::from_millis(2));
        snapshot_save(&data_dir, &file, "v2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        snapshot_save(&data_dir, &file, "v3").unwrap();

        let list = list(&data_dir, &file).unwrap();
        assert_eq!(list.len(), 3);
        // Newest first.
        assert!(list[0].timestamp_ms >= list[1].timestamp_ms);
        assert!(list[1].timestamp_ms >= list[2].timestamp_ms);
    }

    #[test]
    fn prune_drops_oldest() {
        let data_dir = tmp_dir("prune");
        let file = PathBuf::from("/tmp/sample.md");
        for i in 0..5 {
            snapshot_save(&data_dir, &file, &format!("v{i}")).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        prune(&data_dir, &file, 2).unwrap();
        let list = list(&data_dir, &file).unwrap();
        assert_eq!(list.len(), 2);
    }
}
