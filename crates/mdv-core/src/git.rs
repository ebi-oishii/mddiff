use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::diff::{full_diff, line_diff, DiffLine, HunkSummary};

pub const DEFAULT_BASE: &str = "HEAD";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseKind {
    Special,
    Branch,
    Tag,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseOption {
    /// String to pass to git (revparse_single).
    pub revspec: String,
    /// Display label for UI lists.
    pub label: String,
    pub kind: BaseKind,
    /// Optional supplementary info (commit summary, etc.).
    pub detail: Option<String>,
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not in a git repository")]
    NotARepo,
    #[error("unknown revision: {0}")]
    UnknownRevision(String),
    #[error("git operation failed: {0}")]
    Git(#[from] git2::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file is outside the discovered repository")]
    OutsideRepo,
    #[error("blob content is not valid utf-8")]
    NotUtf8,
}

pub fn is_in_repo(file: &Path) -> bool {
    let abs = canonicalize_lossy(file);
    git2::Repository::discover(&abs).is_ok()
}

pub fn diff_against_head(file: &Path) -> Result<Vec<HunkSummary>, GitError> {
    let current = std::fs::read_to_string(file)?;
    diff_text_against_base(file, &current, DEFAULT_BASE)
}

pub fn diff_text_against_base(
    file: &Path,
    current: &str,
    base: &str,
) -> Result<Vec<HunkSummary>, GitError> {
    let old = base_text_for(file, base)?;
    Ok(line_diff(&old, current))
}

pub fn full_diff_against_base(
    file: &Path,
    current: &str,
    base: &str,
) -> Result<Vec<DiffLine>, GitError> {
    let old = base_text_for(file, base)?;
    Ok(full_diff(&old, current))
}

fn base_text_for(file: &Path, base: &str) -> Result<String, GitError> {
    let file_abs = canonicalize_lossy(file);
    let repo = git2::Repository::discover(&file_abs).map_err(|_| GitError::NotARepo)?;
    let workdir = repo.workdir().ok_or(GitError::NotARepo)?;
    let workdir_abs = canonicalize_lossy(workdir);
    let rel = file_abs
        .strip_prefix(&workdir_abs)
        .map_err(|_| GitError::OutsideRepo)?;
    read_revision_blob(&repo, rel, base)
}

fn read_revision_blob(
    repo: &git2::Repository,
    rel: &Path,
    revspec: &str,
) -> Result<String, GitError> {
    let obj = repo
        .revparse_single(revspec)
        .map_err(|_| GitError::UnknownRevision(revspec.to_string()))?;
    let tree = obj.peel_to_tree()?;
    let entry = match tree.get_path(rel) {
        Ok(e) => e,
        Err(_) => return Ok(String::new()),
    };
    let blob = repo.find_blob(entry.id())?;
    std::str::from_utf8(blob.content())
        .map(|s| s.to_string())
        .map_err(|_| GitError::NotUtf8)
}

fn canonicalize_lossy(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

pub fn list_bases(file: &Path) -> Result<Vec<BaseOption>, GitError> {
    let file_abs = canonicalize_lossy(file);
    let repo = git2::Repository::discover(&file_abs).map_err(|_| GitError::NotARepo)?;

    let mut out = vec![
        BaseOption {
            revspec: "HEAD".into(),
            label: "HEAD".into(),
            kind: BaseKind::Special,
            detail: Some("current commit".into()),
        },
        BaseOption {
            revspec: "HEAD~1".into(),
            label: "HEAD~1".into(),
            kind: BaseKind::Special,
            detail: Some("one commit before HEAD".into()),
        },
        BaseOption {
            revspec: "HEAD~5".into(),
            label: "HEAD~5".into(),
            kind: BaseKind::Special,
            detail: Some("five commits before HEAD".into()),
        },
    ];

    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for entry in branches.flatten() {
            let (branch, _) = entry;
            if let Ok(Some(name)) = branch.name() {
                out.push(BaseOption {
                    revspec: name.to_string(),
                    label: name.to_string(),
                    kind: BaseKind::Branch,
                    detail: None,
                });
            }
        }
    }

    if let Ok(tags) = repo.tag_names(None) {
        for tag in tags.iter().flatten() {
            out.push(BaseOption {
                revspec: tag.to_string(),
                label: tag.to_string(),
                kind: BaseKind::Tag,
                detail: None,
            });
        }
    }

    if let Ok(mut walk) = repo.revwalk() {
        let _ = walk.push_head();
        for oid_res in walk.take(15) {
            let Ok(oid) = oid_res else { continue };
            let Ok(commit) = repo.find_commit(oid) else { continue };
            let short = format!("{:.7}", oid);
            let summary: String = commit
                .summary()
                .unwrap_or("")
                .chars()
                .take(70)
                .collect();
            out.push(BaseOption {
                revspec: short.clone(),
                label: format!("{}  {}", short, summary),
                kind: BaseKind::Commit,
                detail: None,
            });
        }
    }

    Ok(out)
}
