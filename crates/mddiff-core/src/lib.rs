pub mod diff;
pub mod doc;
pub mod fs;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod git;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod pack;

pub use diff::{DiffLine, HunkKind, HunkSummary};
pub use doc::DocState;
