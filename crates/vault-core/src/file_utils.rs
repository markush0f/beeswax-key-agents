use std::fs;
use std::path::Path;

use crate::config::EXCLUDED_DIRS;

/// Returns true for any `.env*` file (e.g. `.env`, `.env.local`).
///
/// This is intentionally filename-based (not content-based) to keep it fast and predictable.
pub fn is_env_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with(".env"))
        .unwrap_or(false)
}

pub fn is_scannable_file(path: &Path) -> bool {
    path.is_file()
}

/// Returns true when the directory name is present in `EXCLUDED_DIRS`.
pub fn is_ignored_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(is_excluded_dir_name)
        .unwrap_or(false)
}

pub fn is_excluded_dir_name(name: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|d| d == &name)
}

/// Reads a UTF-8 text file, skipping large or binary files.
///
/// Limits:
/// - Max size: 2 MiB
/// - Binary detection: null byte presence
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
