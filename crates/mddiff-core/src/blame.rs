//! Per-line blame across Git history + local save snapshots.
//!
//! For each line of the current buffer, identify whether it was last touched
//! by a Git commit (we hand back sha / author / date / summary) or by the
//! buffer's local edits (author = "local", date = latest snapshot timestamp
//! if any).
//!
//! Implementation history:
//! - v1 (v0.2.0): used git2's `blame_buffer` to get a single blame that
//!   covered HEAD-derived lines and buffer-only lines in one pass.
//! - v2 (v0.2.3): SEGV reports on certain real files (e.g. CRLF line endings
//!   in HEAD blob, large drift between HEAD and buffer). `blame_buffer` is a
//!   thin wrapper around libgit2's notoriously crashy `git_blame_buffer`.
//!   We now do the alignment ourselves: `blame_file(HEAD)` for canonical
//!   blame, then `full_diff(head_blob, current)` to map current lines back
//!   to HEAD lines, falling through to Local/Buffer for the rest.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::diff::{full_diff, DiffLine};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlameOrigin {
    Git,
    Local,
    Buffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameLine {
    pub line_no: u32,
    pub origin: BlameOrigin,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
    pub author: String,
    pub email: Option<String>,
    pub date_ts: i64,
    pub summary: Option<String>,
}

#[derive(Debug, Error)]
pub enum BlameError {
    #[error("not in a git repository")]
    NotARepo,
    #[error("git operation failed: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file is outside the discovered repository")]
    OutsideRepo,
}

pub fn compute(
    file: &Path,
    current_text: &str,
    latest_snapshot_ts_ms: Option<i64>,
) -> Result<Vec<BlameLine>, BlameError> {
    let file_abs = canonicalize_lossy(file);
    let repo =
        git2::Repository::discover(&file_abs).map_err(|_| BlameError::NotARepo)?;
    let workdir = repo.workdir().ok_or(BlameError::NotARepo)?;
    let workdir_abs = canonicalize_lossy(workdir);
    let rel = file_abs
        .strip_prefix(&workdir_abs)
        .map_err(|_| BlameError::OutsideRepo)?;

    let line_count = count_lines(current_text);
    if line_count == 0 {
        return Ok(Vec::new());
    }

    // HEAD blame (canonical per-line attribution for the HEAD blob).
    let head_blame = match repo.blame_file(rel, None) {
        Ok(b) => b,
        Err(_) => return Ok(all_local(line_count, latest_snapshot_ts_ms)),
    };

    // HEAD blob content. If the file isn't in HEAD's tree at all, the blame
    // came back empty and we already fell through above — but be defensive.
    let head_text = match read_head_blob(&repo, rel) {
        Ok(t) => t,
        Err(_) => return Ok(all_local(line_count, latest_snapshot_ts_ms)),
    };
    // Normalize line endings so `full_diff` aligns CRLF blobs against LF
    // buffers (and vice versa). Without this, every line of a CRLF blob
    // would diff as "different" and the blame would collapse to all-local.
    let head_text = normalize_lf(&head_text);
    let current_norm = normalize_lf(current_text);

    let diff = full_diff(&head_text, &current_norm);

    let mut result = Vec::with_capacity(line_count);
    for dl in diff {
        match dl {
            DiffLine::Equal { old_no, new_no, .. } => {
                let line_no = new_no as u32;
                if let Some(hunk) = head_blame.get_line(old_no) {
                    result.push(git_line(&repo, line_no, hunk.final_commit_id(), &hunk));
                } else {
                    // Unexpected: blame didn't cover this HEAD line.
                    // Fall back to local rather than panic.
                    result.push(local_line(line_no, latest_snapshot_ts_ms));
                }
            }
            DiffLine::Added { new_no, .. } => {
                result.push(local_line(new_no as u32, latest_snapshot_ts_ms));
            }
            DiffLine::Removed { .. } => {
                // Line removed from HEAD; not in current. Skip.
            }
        }
    }

    Ok(result)
}

fn read_head_blob(
    repo: &git2::Repository,
    rel: &Path,
) -> Result<String, git2::Error> {
    let head = repo.head()?.peel_to_tree()?;
    let entry = head.get_path(rel)?;
    let blob = repo.find_blob(entry.id())?;
    // Lossy conversion is OK for blame alignment — the diff just needs
    // stable line boundaries, not byte-exact content. Non-UTF-8 source
    // would have crashed `blame_buffer`; here we degrade gracefully.
    Ok(String::from_utf8_lossy(blob.content()).into_owned())
}

fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn git_line(
    repo: &git2::Repository,
    line_no: u32,
    oid: git2::Oid,
    hunk: &git2::BlameHunk<'_>,
) -> BlameLine {
    if oid.is_zero() {
        // Defensive: in theory HEAD blame should never have null commits
        // (those only appear from blame_buffer's working-tree overlay),
        // but if they do, treat as local instead of building a BlameLine
        // with empty sha.
        return BlameLine {
            line_no,
            origin: BlameOrigin::Buffer,
            sha: None,
            short_sha: None,
            author: "local".to_string(),
            email: None,
            date_ts: 0,
            summary: None,
        };
    }
    let sig = hunk.final_signature();
    let author = sig.name().unwrap_or("").to_string();
    let email = sig.email().map(|s| s.to_string());
    let date_ts = sig.when().seconds();
    let sha = oid.to_string();
    let short_sha = sha.chars().take(8).collect::<String>();
    let summary = repo
        .find_commit(oid)
        .ok()
        .and_then(|c| c.summary().map(|s| s.to_string()));
    BlameLine {
        line_no,
        origin: BlameOrigin::Git,
        sha: Some(sha),
        short_sha: Some(short_sha),
        author,
        email,
        date_ts,
        summary,
    }
}

fn local_line(line_no: u32, latest_snapshot_ts_ms: Option<i64>) -> BlameLine {
    BlameLine {
        line_no,
        origin: if latest_snapshot_ts_ms.is_some() {
            BlameOrigin::Local
        } else {
            BlameOrigin::Buffer
        },
        sha: None,
        short_sha: None,
        author: "local".to_string(),
        email: None,
        date_ts: latest_snapshot_ts_ms.map(|ms| ms / 1000).unwrap_or(0),
        summary: None,
    }
}

fn all_local(count: usize, latest_snapshot_ts_ms: Option<i64>) -> Vec<BlameLine> {
    (1..=count)
        .map(|n| local_line(n as u32, latest_snapshot_ts_ms))
        .collect()
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count()
    }
}

fn canonicalize_lossy(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}
