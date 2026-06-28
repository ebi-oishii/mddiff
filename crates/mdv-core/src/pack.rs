//! `.mdv` package format v1: pack a Markdown file plus its Git history into
//! a single self-contained document.
//!
//! See `docs/mdv-protocol.md` for the on-disk format. This module implements:
//! - `pack`: take a file's current text + a base commit, walk Git from HEAD
//!   down to base (inclusive), bundle all touched snapshots + commits + the
//!   current text as a virtual head, zstd-compress, base64-encode, wrap in a
//!   `<!-- mdv:v1 ... -->` HTML comment and append to the body.
//! - `extract_body`: strip the package block back out so the body can be
//!   loaded as plain Markdown on import.
//!
//! v1 deviations from the spec:
//! - JSON serialization uses serde_json with `BTreeMap` for sorted keys
//!   instead of strict RFC 8785 JCS. The result is byte-stable for our own
//!   reader and round-trips correctly, but other strict-JCS implementations
//!   could differ on Unicode escaping / number formatting.
//! - `repo_id` is deterministic from the file's canonical path (not random),
//!   so re-packing the same file always yields the same `repo_id`. Convenient
//!   for "register" semantics where successive exports describe the same logical
//!   document.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const FORMAT: &str = "mdv-bundle";
const VERSION: u32 = 1;
const COMMIT_HASH_PREFIX: &[u8] = b"mdv.commit.v1\0";
const PACKAGE_OPEN: &str = "<!-- mdv:v1";
const PACKAGE_CLOSE: &str = "-->";

#[derive(Debug, Error)]
pub enum PackError {
    #[error("not in a git repository")]
    NotARepo,
    #[error("file is outside the discovered repository")]
    OutsideRepo,
    #[error("unknown base revision: {0}")]
    UnknownRevision(String),
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("blob at {0} is not utf-8")]
    NotUtf8(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub format: String,
    pub version: u32,
    pub repo_id: String,
    pub head: String,
    pub refs: BTreeMap<String, String>,
    pub commits: BTreeMap<String, CommitNode>,
    pub snapshots: BTreeMap<String, SnapshotNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitNode {
    #[serde(rename = "type")]
    pub kind: String, // "commit"
    pub parents: Vec<String>,
    pub body: String, // snapshot ID
    pub author: Author,
    pub created_at: String,
    pub message: String,
    pub source: SourceInfo,
    pub signatures: Vec<String>, // empty for v1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceInfo {
    pub app: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotNode {
    #[serde(rename = "type")]
    pub kind: String, // "snapshot"
    pub size: usize,
    pub text: String,
}

/// Result of a pack call: the full `.mdv` content (markdown body + package
/// block) and a small set of stats for the UI.
#[derive(Debug)]
pub struct PackResult {
    pub content: String,
    pub commit_count: usize,
    pub snapshot_count: usize,
    pub bundle_bytes: usize,
}

fn canonicalize_lossy(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
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

fn snapshot_id(body_canonical: &str) -> String {
    format!("b:{}", sha256_hex(body_canonical.as_bytes()))
}

fn commit_id(commit_without_signatures: &CommitNode) -> Result<String, PackError> {
    // Serialize without signatures using BTreeMap for stable key order.
    let json = serde_json::to_string(commit_without_signatures)?;
    let mut buf = Vec::with_capacity(COMMIT_HASH_PREFIX.len() + json.len());
    buf.extend_from_slice(COMMIT_HASH_PREFIX);
    buf.extend_from_slice(json.as_bytes());
    Ok(format!("c:{}", sha256_hex(&buf)))
}

/// Deterministic repo_id derived from the file's canonical path. Two exports
/// of the same file produce the same id, so consumers can recognize them as
/// the same logical document.
fn repo_id_for(file_abs: &Path) -> String {
    let bytes = file_abs.as_os_str().as_encoded_bytes();
    let digest = sha256_hex(bytes);
    // Format as a urn:uuid using the first 16 bytes of the hash (not RFC 4122
    // version-compliant — purely an identifier scheme).
    format!(
        "urn:uuid:{}-{}-{}-{}-{}",
        &digest[0..8],
        &digest[8..12],
        &digest[12..16],
        &digest[16..20],
        &digest[20..32],
    )
}

/// Pack the file's Git history (or a slice of it, anchored at `base_revspec`)
/// plus the current buffer contents into a `.mdv` bundle.
///
/// The synthetic "Packaged" commit that represents the current buffer carries
/// `author_name` / `author_email` in its metadata so receivers can see who
/// produced the bundle. Callers should source these from the Git repo's
/// `user.name` / `user.email` config (or whatever identity is appropriate to
/// the host). Pass `"Unknown"` for the name if no identity is available;
/// never hardcode a maintainer handle.
pub fn pack(
    file: &Path,
    current_text: &str,
    base_revspec: &str,
    author_name: &str,
    author_email: Option<&str>,
) -> Result<PackResult, PackError> {
    let file_abs = canonicalize_lossy(file);
    let repo = git2::Repository::discover(&file_abs).map_err(|_| PackError::NotARepo)?;
    let workdir = repo.workdir().ok_or(PackError::NotARepo)?;
    let workdir_abs = canonicalize_lossy(workdir);
    let rel: PathBuf = file_abs
        .strip_prefix(&workdir_abs)
        .map_err(|_| PackError::OutsideRepo)?
        .to_path_buf();

    let base_oid = repo
        .revparse_single(base_revspec)
        .map_err(|_| PackError::UnknownRevision(base_revspec.to_string()))?
        .peel_to_commit()?
        .id();

    // Walk newest → oldest; stop after we've included `base_oid`.
    let head_commit = repo.head()?.peel_to_commit()?;
    let mut walker = repo.revwalk()?;
    walker.push(head_commit.id())?;

    let mut walk_oids: Vec<git2::Oid> = Vec::new();
    for oid in walker {
        let oid = oid?;
        walk_oids.push(oid);
        if oid == base_oid {
            break;
        }
    }
    // Build oldest-first so child commits can reference already-computed parent c: IDs.
    walk_oids.reverse();
    let included: std::collections::HashSet<git2::Oid> = walk_oids.iter().copied().collect();

    let mut snapshots: BTreeMap<String, SnapshotNode> = BTreeMap::new();
    let mut commits: BTreeMap<String, CommitNode> = BTreeMap::new();
    // git oid → our c: id for ancestors that are in our walk
    let mut git_to_cid: std::collections::HashMap<git2::Oid, String> = Default::default();

    for oid in &walk_oids {
        let commit = repo.find_commit(*oid)?;
        // Skip commits where the file doesn't exist at this revision — they
        // can't contribute a snapshot.
        let tree = commit.tree()?;
        let entry = match tree.get_path(&rel) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let blob = repo.find_blob(entry.id())?;
        let raw = std::str::from_utf8(blob.content())
            .map_err(|_| PackError::NotUtf8(rel.display().to_string()))?;
        let body = normalize_lf(raw);
        let snap_id = snapshot_id(&body);
        snapshots.entry(snap_id.clone()).or_insert(SnapshotNode {
            kind: "snapshot".into(),
            size: body.len(),
            text: body,
        });

        // parents: only those whose c: id we've already computed; commits that
        // walked off the back of our window become "root" for the bundle.
        let parents: Vec<String> = commit
            .parents()
            .filter_map(|p| {
                if included.contains(&p.id()) {
                    git_to_cid.get(&p.id()).cloned()
                } else {
                    None
                }
            })
            .collect();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let created_at = Utc
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".into());

        let node = CommitNode {
            kind: "commit".into(),
            parents,
            body: snap_id,
            author: Author {
                name: author_name,
                email: None, // strip personal info per project convention
            },
            created_at,
            message: commit.summary().unwrap_or("").to_string(),
            source: SourceInfo {
                app: "mdv".into(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            signatures: Vec::new(),
        };

        let cid = commit_id(&node)?;
        git_to_cid.insert(*oid, cid.clone());
        commits.insert(cid, node);
    }

    // Virtual head commit representing the current (possibly uncommitted) buffer.
    let current_canonical = normalize_lf(current_text);
    let current_snap_id = snapshot_id(&current_canonical);
    snapshots
        .entry(current_snap_id.clone())
        .or_insert(SnapshotNode {
            kind: "snapshot".into(),
            size: current_canonical.len(),
            text: current_canonical,
        });

    let head_parent = git_to_cid.get(&head_commit.id()).cloned();
    let head_node = CommitNode {
        kind: "commit".into(),
        parents: head_parent.into_iter().collect(),
        body: current_snap_id,
        author: Author {
            name: author_name.into(),
            email: author_email.map(|s| s.to_string()),
        },
        created_at: Utc::now().to_rfc3339(),
        message: "Packaged".into(),
        source: SourceInfo {
            app: "mdv".into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        signatures: Vec::new(),
    };
    let head_cid = commit_id(&head_node)?;
    let mut refs = BTreeMap::new();
    refs.insert("main".to_string(), head_cid.clone());
    commits.insert(head_cid.clone(), head_node);

    let bundle = Bundle {
        format: FORMAT.into(),
        version: VERSION,
        repo_id: repo_id_for(&file_abs),
        head: head_cid,
        refs,
        commits,
        snapshots,
    };

    let canonical = serde_json::to_string(&bundle)?;
    let canonical_bytes = canonical.as_bytes();
    let compressed = zstd::encode_all(canonical_bytes, 19)?;
    let payload = base64::engine::general_purpose::STANDARD.encode(&compressed);

    let block = format!(
        "{}\ncodec: zstd+base64\nbundle-sha256: sha256:{}\nbundle-size: {}\npayload:\n{}\n{}",
        PACKAGE_OPEN,
        sha256_hex(canonical_bytes),
        canonical_bytes.len(),
        wrap_base64(&payload, 76),
        PACKAGE_CLOSE,
    );

    let body = normalize_lf(current_text);
    let body_trimmed = body.trim_end_matches('\n');
    let content = format!("{}\n\n{}\n", body_trimmed, block);

    Ok(PackResult {
        content,
        commit_count: bundle.commits.len(),
        snapshot_count: bundle.snapshots.len(),
        bundle_bytes: compressed.len(),
    })
}

fn wrap_base64(s: &str, width: usize) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / width);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % width == 0 {
            out.push('\n');
        }
        out.push(c);
    }
    out
}

/// Returns the markdown body with any trailing `<!-- mdv:v1 ... -->` package
/// block stripped. If no package block is found, returns the input unchanged.
pub fn extract_body(content: &str) -> String {
    let Some(start) = content.rfind(PACKAGE_OPEN) else {
        return content.to_string();
    };
    let after_start = &content[start..];
    let Some(end_rel) = after_start.find(PACKAGE_CLOSE) else {
        return content.to_string();
    };
    let end = start + end_rel + PACKAGE_CLOSE.len();
    let mut body = String::with_capacity(start + (content.len() - end));
    body.push_str(content[..start].trim_end_matches(['\n', ' ', '\t']));
    body.push_str(content[end..].trim_start_matches(['\n', ' ', '\t']));
    let trimmed = body.trim_end_matches('\n').to_string();
    format!("{}\n", trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_body_strips_trailing_package() {
        let input = "# Hello\n\nbody\n\n<!-- mdv:v1\ncodec: x\npayload:\nAAAA\n-->\n";
        let out = extract_body(input);
        assert_eq!(out, "# Hello\n\nbody\n");
    }

    #[test]
    fn extract_body_leaves_plain_md_alone() {
        let input = "# Hello\n\nbody\n";
        assert_eq!(extract_body(input), input);
    }

    #[test]
    fn extract_body_handles_no_trailing_newline() {
        let input = "x\n<!-- mdv:v1\npayload:\nAAA\n-->";
        let out = extract_body(input);
        assert_eq!(out, "x\n");
    }
}
