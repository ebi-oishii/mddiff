use serde::{Deserialize, Serialize};
use similar::{ChangeTag, DiffOp, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HunkKind {
    Added,
    Modified,
    Removed,
}

/// Descriptor of a single change, carrying line ranges on both the OLD and
/// NEW sides. All line numbers are 1-based. A `point` (anchor) is encoded as
/// `start == end` (the line immediately before insertion / deletion, or 0
/// for top-of-file).
///
/// Conventions per `kind`:
/// - `Added`    : `new_*` spans the inserted lines; `old_start == old_end`
///                points to where in OLD they would have been inserted.
/// - `Removed`  : `old_*` spans the deleted lines; `new_start == new_end`
///                points to where in NEW the deletion occurred.
/// - `Modified` : both `new_*` and `old_*` span the replaced lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HunkSummary {
    pub kind: HunkKind,
    pub new_start: usize,
    pub new_end: usize,
    pub old_start: usize,
    pub old_end: usize,
}

impl HunkSummary {
    /// Number of OLD-side lines covered (deleted or replaced). 0 for pure Added.
    pub fn removed_count(&self) -> usize {
        match self.kind {
            HunkKind::Added => 0,
            HunkKind::Removed | HunkKind::Modified => {
                self.old_end.saturating_sub(self.old_start) + 1
            }
        }
    }

    /// Number of NEW-side lines covered (added or replaced). 0 for pure Removed.
    pub fn added_count(&self) -> usize {
        match self.kind {
            HunkKind::Removed => 0,
            HunkKind::Added | HunkKind::Modified => {
                self.new_end.saturating_sub(self.new_start) + 1
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DiffLine {
    Equal {
        old_no: usize,
        new_no: usize,
        text: String,
    },
    Added {
        new_no: usize,
        text: String,
    },
    Removed {
        old_no: usize,
        text: String,
    },
}

pub fn line_diff(old: &str, new: &str) -> Vec<HunkSummary> {
    let diff = TextDiff::from_lines(old, new);
    let mut hunks = Vec::new();

    for op in diff.ops().iter().copied() {
        match op {
            DiffOp::Equal { .. } => {}
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => hunks.push(HunkSummary {
                kind: HunkKind::Added,
                new_start: new_index + 1,
                new_end: new_index + new_len,
                old_start: old_index,
                old_end: old_index,
            }),
            DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => hunks.push(HunkSummary {
                kind: HunkKind::Removed,
                new_start: new_index,
                new_end: new_index,
                old_start: old_index + 1,
                old_end: old_index + old_len,
            }),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => hunks.push(HunkSummary {
                kind: HunkKind::Modified,
                new_start: new_index + 1,
                new_end: new_index + new_len,
                old_start: old_index + 1,
                old_end: old_index + old_len,
            }),
        }
    }

    hunks
}

pub fn full_diff(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old, new);
    let mut out = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change
            .value()
            .strip_suffix('\n')
            .unwrap_or(change.value())
            .to_string();
        match change.tag() {
            ChangeTag::Equal => out.push(DiffLine::Equal {
                old_no: change.old_index().unwrap() + 1,
                new_no: change.new_index().unwrap() + 1,
                text,
            }),
            ChangeTag::Insert => out.push(DiffLine::Added {
                new_no: change.new_index().unwrap() + 1,
                text,
            }),
            ChangeTag::Delete => out.push(DiffLine::Removed {
                old_no: change.old_index().unwrap() + 1,
                text,
            }),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_added_line() {
        let old = "a\nb\n";
        let new = "a\nb\nc\n";
        let hunks = line_diff(old, new);
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(h.kind, HunkKind::Added);
        assert_eq!((h.new_start, h.new_end), (3, 3));
        // Insertion happens after OLD line 2 → anchor = 2 (point: start == end).
        assert_eq!((h.old_start, h.old_end), (2, 2));
        assert_eq!(h.added_count(), 1);
        assert_eq!(h.removed_count(), 0);
    }

    #[test]
    fn detects_removed_line() {
        let old = "a\nb\nc\n";
        let new = "a\nc\n";
        let hunks = line_diff(old, new);
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(h.kind, HunkKind::Removed);
        // Deletion in OLD covers line 2.
        assert_eq!((h.old_start, h.old_end), (2, 2));
        // In NEW it sits after line 1 (anchor 1).
        assert_eq!((h.new_start, h.new_end), (1, 1));
        assert_eq!(h.removed_count(), 1);
        assert_eq!(h.added_count(), 0);
    }

    #[test]
    fn detects_modified_line() {
        let old = "a\nb\nc\n";
        let new = "a\nB\nc\n";
        let hunks = line_diff(old, new);
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(h.kind, HunkKind::Modified);
        assert_eq!((h.new_start, h.new_end), (2, 2));
        assert_eq!((h.old_start, h.old_end), (2, 2));
        assert_eq!(h.added_count(), 1);
        assert_eq!(h.removed_count(), 1);
    }

    #[test]
    fn detects_multi_line_replacement() {
        let old = "a\nb\nc\nd\n";
        let new = "a\nX\nY\nZ\nd\n";
        let hunks = line_diff(old, new);
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(h.kind, HunkKind::Modified);
        assert_eq!((h.new_start, h.new_end), (2, 4));
        assert_eq!((h.old_start, h.old_end), (2, 3));
        assert_eq!(h.added_count(), 3);
        assert_eq!(h.removed_count(), 2);
    }

    #[test]
    fn unchanged_yields_no_hunks() {
        let s = "a\nb\nc\n";
        assert!(line_diff(s, s).is_empty());
    }

    #[test]
    fn full_diff_marks_each_line() {
        let old = "a\nb\n";
        let new = "a\nB\n";
        let lines = full_diff(old, new);
        assert!(matches!(lines[0], DiffLine::Equal { .. }));
        assert!(matches!(lines[1], DiffLine::Removed { .. }));
        assert!(matches!(lines[2], DiffLine::Added { .. }));
    }
}
