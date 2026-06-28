pub mod fs;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod diff;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod git;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod mdv;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod watcher;
