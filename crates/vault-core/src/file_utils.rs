//! File system utility predicates for the scanner pipeline.
//!
//! This module centralises all path-based decisions made during directory walks:
//! whether a file is an environment config, whether a directory should be skipped,
//! and whether a file is safe to read as UTF-8 text. Keeping these predicates here
//! makes the scanning logic in [`crate::scan`] easier to read and test independently.

use std::fs;
use std::path::Path;

use crate::config::EXCLUDED_DIRS;

/// Returns `true` if the given path points to a `.env*` file.
///
/// Detection is purely filename-based (not content-based) for speed and
/// predictability. Any file whose name starts with `.env` qualifies, including
/// `.env`, `.env.local`, `.env.production`, etc.
///
/// Returns `false` for directories or paths with no file name component.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use vault_core::config; // exposed for testing purposes
/// // Internal to the crate — used by scan.rs
/// ```
pub fn is_env_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with(".env"))
        .unwrap_or(false)
}

/// Returns `true` if the given path is a regular file.
///
/// This is the minimal gate for including a path in a file scan. Binary files
/// and oversized files are filtered out later by [`read_text_file`].
///
/// In the future this predicate could be extended to filter by extension allowlist
/// without breaking the scanning call sites.
pub fn is_scannable_file(path: &Path) -> bool {
    path.is_file()
}

/// Returns `true` if the given directory path should be skipped during a recursive walk.
///
/// A directory is ignored when its name (not its full path) appears in [`EXCLUDED_DIRS`].
/// This covers VCS metadata (`.git`), build artifacts (`target`, `dist`), package
/// manager stores (`node_modules`), and many more tooling-specific directories.
///
/// Returns `false` for files and for paths with no obtainable directory name.
///
/// # Note
///
/// This predicate is used with [`walkdir::WalkDir::filter_entry`], meaning it
/// prunes entire subtrees — not just individual entries.
pub fn is_ignored_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(is_excluded_dir_name)
        .unwrap_or(false)
}

/// Returns `true` if `name` is present in the [`EXCLUDED_DIRS`] denylist.
///
/// This is a pure string lookup used internally by [`is_ignored_dir`].
pub fn is_excluded_dir_name(name: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|d| d == &name)
}

/// Reads a file as a UTF-8 string, skipping files that are too large or binary.
///
/// Two safety checks are applied before the file content is returned:
///
/// 1. **Size limit**: Files larger than **2 MiB** are skipped. This avoids loading
///    generated assets, database dumps, or log archives into memory.
/// 2. **Binary detection**: If the file bytes contain a null byte (`\0`), the file
///    is treated as binary and skipped. This is a fast heuristic that rejects compiled
///    objects, images, archives, and other non-text formats while remaining reliable
///    enough for the scanner's use case.
///
/// Returns `None` if the file cannot be read, is too large, is binary, or is not
/// valid UTF-8.
///
/// # Arguments
///
/// * `path` - Path to the file to read.
///
/// # Performance
///
/// The null-byte check reads the entire file into memory once. Consider adding a
/// streaming check if the size limit is raised significantly in the future.
pub fn read_text_file(path: &Path) -> Option<String> {
    const MAX_FILE_SIZE_BYTES: u64 = 2 * 1024 * 1024;

    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_FILE_SIZE_BYTES {
        return None;
    }

    let bytes = fs::read(path).ok()?;
    if bytes.contains(&0) {
        return None;
    }

    String::from_utf8(bytes).ok()
}
