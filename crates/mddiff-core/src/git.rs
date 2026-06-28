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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffMarker {
    /// File at this revision differs from current buffer AND this is the
    /// canonical (oldest in a same-content run) option.
    Differs,
    /// File at this revision is byte-identical to the current buffer.
    Identical,
    /// File at this revision has the same content as a newer entry shown
    /// above in the list; the newer entry is the canonical comparison
    /// point, so this one is suppressed.
    Redundant,
    /// Current buffer was not provided, so we can't compare against it.
    /// Note: a revspec that doesn't resolve at all is marked `Differs`, not
    /// `Unknown` — see the match in `list_base_options_for_file`.
    Unknown,
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
    pub marker: DiffMarker,
    /// True iff this revision changed the file's blob compared to its parent.
    /// Always `true` for Special / Branch / Tag (we don't filter them out).
    /// For Commit kind, this is what lets the UI hide "noise" commits that
    /// didn't touch this file from the default picker.
    pub file_changed: bool,
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

/// Read `user.name` / `user.email` from the Git config that applies to
/// `file`. Returns `("Unknown", None)` when the file isn't in a Git repo or
/// the user identity isn't set — never hardcoded to a maintainer handle.
/// Repo-level config wins over the global, matching how `git commit`
/// resolves identity. Used by `.mddiff` pack to stamp the synthetic head commit.
pub fn user_identity(file: &Path) -> (String, Option<String>) {
    let abs = canonicalize_lossy(file);
    if let Ok(repo) = git2::Repository::discover(&abs) {
        if let Ok(cfg) = repo.config() {
            let name = cfg.get_string("user.name").ok();
            let email = cfg.get_string("user.email").ok();
            return (name.unwrap_or_else(|| "Unknown".to_string()), email);
        }
    }
    ("Unknown".to_string(), None)
}

pub fn diff_against_head(file: &Path) -> Result<Vec<HunkSummary>, GitError> {
    let current = std::fs::read_to_string(file)?;
    diff_text_against_base(file, &current, DEFAULT_BASE)
}

pub fn read_at(file: &Path, revspec: &str) -> Result<String, GitError> {
    base_text_for(file, revspec)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideBySidePayload {
    pub old_text: String,
    pub new_text: String,
    pub hunks: Vec<HunkSummary>,
}

/// Bundle of OLD blob + NEW buffer + hunks, for rendering a 2-pane Side-by-Side
/// view in a single IPC round trip.
pub fn side_by_side_against_base(
    file: &Path,
    current: &str,
    base: &str,
) -> Result<SideBySidePayload, GitError> {
    let old = base_text_for(file, base)?;
    let hunks = line_diff(&old, current);
    Ok(SideBySidePayload {
        old_text: old,
        new_text: current.to_string(),
        hunks,
    })
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

/// Enumerate possible diff bases for `file`. If `current` is provided, each
/// option's `marker` field describes how it relates to the current buffer:
/// - `Differs`: file at this revision differs from current buffer AND is the
///   canonical entry of its same-content run (see Recent-commits dedup below)
/// - `Identical`: file at this revision is byte-identical to the current buffer
/// - `Redundant`: a more recent commit shown above in the list already has the
///   same file content; choosing this one would produce the same diff
/// - `Unknown`: `current` was None, or the revision couldn't be resolved
///
/// Dedup behavior: within the Recent-commits group, when adjacent entries
/// (newest-to-oldest) share the same blob OID and both differ from the buffer,
/// the newer one is downgraded to `Redundant` so only the OLDEST commit in the
/// same-content run keeps the `Differs` marker. Cross-kind dedup is not
/// performed (Special / Branches / Tags use their own canonical names).
pub fn list_bases(file: &Path, current: Option<&str>) -> Result<Vec<BaseOption>, GitError> {
    let file_abs = canonicalize_lossy(file);
    let repo = git2::Repository::discover(&file_abs).map_err(|_| GitError::NotARepo)?;
    let workdir = repo.workdir().ok_or(GitError::NotARepo)?;
    let workdir_abs = canonicalize_lossy(workdir);
    let rel = file_abs
        .strip_prefix(&workdir_abs)
        .map_err(|_| GitError::OutsideRepo)?
        .to_path_buf();

    let current_oid = current
        .and_then(|c| git2::Oid::hash_object(git2::ObjectType::Blob, c.as_bytes()).ok());

    let mk = |revspec: String,
              label: String,
              kind: BaseKind,
              detail: Option<String>,
              file_changed: bool| {
        let blob = blob_oid_at(&repo, &rel, &revspec);
        let marker = match (current_oid, blob) {
            (None, _) => DiffMarker::Unknown,
            (Some(_), None) => DiffMarker::Differs,
            (Some(cur), Some(b)) if cur == b => DiffMarker::Identical,
            (Some(_), Some(_)) => DiffMarker::Differs,
        };
        let opt = BaseOption {
            revspec,
            label,
            kind,
            detail,
            marker,
            file_changed,
        };
        (blob, opt)
    };

    let mut entries: Vec<(Option<git2::Oid>, BaseOption)> = vec![
        mk(
            "HEAD".into(),
            "HEAD".into(),
            BaseKind::Special,
            Some("current commit".into()),
            true,
        ),
        mk(
            "HEAD~1".into(),
            "HEAD~1".into(),
            BaseKind::Special,
            Some("one commit before HEAD".into()),
            true,
        ),
        mk(
            "HEAD~5".into(),
            "HEAD~5".into(),
            BaseKind::Special,
            Some("five commits before HEAD".into()),
            true,
        ),
    ];

    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for entry in branches.flatten() {
            let (branch, _) = entry;
            if let Ok(Some(name)) = branch.name() {
                entries.push(mk(
                    name.to_string(),
                    name.to_string(),
                    BaseKind::Branch,
                    None,
                    true,
                ));
            }
        }
    }

    if let Ok(tags) = repo.tag_names(None) {
        for tag in tags.iter().flatten() {
            entries.push(mk(
                tag.to_string(),
                tag.to_string(),
                BaseKind::Tag,
                None,
                true,
            ));
        }
    }

    // Walk depth of 50 (was 15). At 15, a stretch of commits that didn't
    // touch the file would push the actual file-changing commits out of the
    // window, making them invisible in the picker. 50 covers most everyday
    // history while keeping the work bounded.
    if let Ok(mut walk) = repo.revwalk() {
        let _ = walk.push_head();
        for oid_res in walk.take(50) {
            let Ok(oid) = oid_res else { continue };
            let Ok(commit) = repo.find_commit(oid) else { continue };
            let short = format!("{:.7}", oid);
            let summary: String = commit
                .summary()
                .unwrap_or("")
                .chars()
                .take(70)
                .collect();

            // file_changed: this commit's blob for `rel` differs from its
            // first parent's blob. Used by the UI to hide commits that
            // didn't touch this file from the default picker.
            let commit_blob = commit
                .tree()
                .ok()
                .and_then(|t| t.get_path(&rel).ok().map(|e| e.id()));
            let parent_blob = commit
                .parent(0)
                .ok()
                .and_then(|p| p.tree().ok())
                .and_then(|t| t.get_path(&rel).ok().map(|e| e.id()));
            let file_changed = commit_blob != parent_blob;

            entries.push(mk(
                short.clone(),
                format!("{}  {}", short, summary),
                BaseKind::Commit,
                None,
                file_changed,
            ));
        }
    }

    dedup_commit_run(&mut entries);

    Ok(entries.into_iter().map(|(_, o)| o).collect())
}

/// Within a Recent-commits run (list order newest → oldest), demote the newer
/// of two adjacent `Differs` entries with the same blob to `Redundant`.
/// Chains naturally for runs of 3+ commits.
fn dedup_commit_run<T: PartialEq + Copy>(entries: &mut [(Option<T>, BaseOption)]) {
    let mut prev: Option<(usize, T)> = None;
    for i in 0..entries.len() {
        if entries[i].1.kind != BaseKind::Commit {
            prev = None;
            continue;
        }
        if let Some(blob) = entries[i].0 {
            if let Some((pi, pb)) = prev {
                if pb == blob
                    && entries[i].1.marker == DiffMarker::Differs
                    && entries[pi].1.marker == DiffMarker::Differs
                {
                    entries[pi].1.marker = DiffMarker::Redundant;
                }
            }
            prev = Some((i, blob));
        } else {
            prev = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(kind: BaseKind, marker: DiffMarker, label: &str) -> BaseOption {
        BaseOption {
            revspec: label.into(),
            label: label.into(),
            kind,
            detail: None,
            marker,
            file_changed: true,
        }
    }

    #[test]
    fn dedup_keeps_only_oldest_of_run() {
        // Newer → older: HEAD, HEAD~1 share blob 1; HEAD~2, HEAD~3, HEAD~4 share
        // blob 2; HEAD~5 alone with blob 3.
        let mut entries: Vec<(Option<u32>, BaseOption)> = vec![
            (Some(1), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD")),
            (Some(1), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD~1")),
            (Some(2), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD~2")),
            (Some(2), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD~3")),
            (Some(2), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD~4")),
            (Some(3), mk(BaseKind::Commit, DiffMarker::Differs, "HEAD~5")),
        ];
        dedup_commit_run(&mut entries);
        let markers: Vec<DiffMarker> = entries.iter().map(|(_, o)| o.marker).collect();
        assert_eq!(
            markers,
            vec![
                DiffMarker::Redundant, // HEAD (newer of {HEAD, HEAD~1})
                DiffMarker::Differs,   // HEAD~1 (oldest of run, kept)
                DiffMarker::Redundant, // HEAD~2
                DiffMarker::Redundant, // HEAD~3
                DiffMarker::Differs,   // HEAD~4 (oldest of run of 3, kept)
                DiffMarker::Differs,   // HEAD~5 (alone)
            ]
        );
    }

    #[test]
    fn dedup_ignores_identical_runs() {
        let mut entries: Vec<(Option<u32>, BaseOption)> = vec![
            (Some(1), mk(BaseKind::Commit, DiffMarker::Identical, "HEAD")),
            (Some(1), mk(BaseKind::Commit, DiffMarker::Identical, "HEAD~1")),
        ];
        dedup_commit_run(&mut entries);
        // Both remain Identical (the buffer matches both); dedup only acts on
        // Differs runs.
        assert_eq!(entries[0].1.marker, DiffMarker::Identical);
        assert_eq!(entries[1].1.marker, DiffMarker::Identical);
    }

    #[test]
    fn dedup_resets_at_non_commit_kind() {
        let mut entries: Vec<(Option<u32>, BaseOption)> = vec![
            (Some(1), mk(BaseKind::Commit, DiffMarker::Differs, "c1")),
            (Some(1), mk(BaseKind::Branch, DiffMarker::Differs, "main")),
            (Some(1), mk(BaseKind::Commit, DiffMarker::Differs, "c2")),
        ];
        dedup_commit_run(&mut entries);
        // No commit adjacency: c1 and c2 are both alone in their commit-runs.
        assert_eq!(entries[0].1.marker, DiffMarker::Differs);
        assert_eq!(entries[1].1.marker, DiffMarker::Differs);
        assert_eq!(entries[2].1.marker, DiffMarker::Differs);
    }
}

fn blob_oid_at(repo: &git2::Repository, rel: &Path, revspec: &str) -> Option<git2::Oid> {
    let obj = repo.revparse_single(revspec).ok()?;
    let tree = obj.peel_to_tree().ok()?;
    tree.get_path(rel).ok().map(|entry| entry.id())
}
