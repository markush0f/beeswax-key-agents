//! High-level scanning entry points for the secret detection pipeline.
//!
//! Each public function in this module represents a distinct scanning strategy
//! tailored to a specific part of a project's file system layout:
//!
//! | Function | Scope |
//! |---|---|
//! | [`scan_env_for_keys`] | `.env*` files only |
//! | [`scan_env_for_keys_streaming`] | `.env*` files, streaming |
//! | [`scan_all_files_for_keys`] | All text files in the tree |
//! | [`scan_all_files_for_keys_streaming`] | All text files, streaming |
//! | [`scan_project_files_for_keys_streaming`] | Source code only (excludes `.env*`) |
//! | [`scan_ide_files_for_keys_streaming`] | IDE config dirs only (`.vscode`, `.idea`) |
//!
//! ## Caching
//!
//! All functions transparently load and write a BLAKE3-based file cache stored under
//! `.vault-cache/index.json` relative to the scanned root. On the first run every
//! file is processed; on subsequent runs only changed files are re-scanned.
//!
//! ## Streaming vs. Collecting
//!
//! Prefer the `_streaming` variants for large repositories to avoid allocating
//! a `Vec` large enough to hold all matches. The collecting wrappers (`scan_env_for_keys`,
//! `scan_all_files_for_keys`) are convenience helpers for tests and small scans.

use std::path::Path;

use walkdir::WalkDir;

use crate::cache::{Cache, CacheMatch};
use crate::config::IDE_DIRS;
use crate::file_utils::{is_env_file, is_ignored_dir, is_scannable_file, read_text_file};
use crate::matcher::find_matches_in_content_streaming_with_hash;
use crate::patterns::get_patterns;
use crate::types::KeyMatch;

/// Scans `path` recursively for `.env*` files and collects all discovered secrets.
///
/// This is a convenience wrapper around [`scan_env_for_keys_streaming`] that
/// materialises all matches into a `Vec`. For large repositories or real-time UIs,
/// use the streaming variant to process matches as they are found.
///
/// # Arguments
///
/// * `path` - Root directory (or file path) to start the walk from.
///
/// # Returns
///
/// A `Vec<KeyMatch>` containing every secret found across all `.env*` files under `path`.
pub fn scan_env_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_env_for_keys_streaming(path, |m| matches.push(m));
    matches
}

/// Scans `path` recursively for `.env*` files, emitting matches via a callback.
///
/// All secrets found in environment files are treated as **hardcoded** unconditionally —
/// the literal assignment `KEY=value` is already the source of truth regardless of
/// whether the value looks like a placeholder.
///
/// The scan leverages the file content cache: if a file's BLAKE3 hash has not changed
/// since the last run, its previously discovered matches are re-emitted instantly
/// without re-reading the file.
///
/// # Arguments
///
/// * `path` - Root directory (or file path) to start scanning from.
/// * `on_match` - Closure called once for every discovered [`KeyMatch`].
pub fn scan_env_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| is_env_file(p))
        .filter_map(|p| read_text_file(&p).map(|content| (p, content)))
        .for_each(|(p, content)| {
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            if let Some(cached) = cache.get_matches_with_hash(&p, &content_hash) {
                emit_cached_matches(&p, cached, &mut on_match);
                return;
            }

            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                &p,
                &content,
                &patterns_list,
                true,
                &mut |m, key_hash| {
                    cached_matches.push(CacheMatch {
                        provider: m.provider.clone(),
                        line_number: m.line_number,
                        key_masked: m.key.clone(),
                        hardcoded: m.hardcoded,
                        key_hash,
                    });
                    on_match(m);
                },
            );
            cache.store(&p, content_hash, cached_matches);
        });

    cache.save();
}

/// Scans all readable text files under `path` and collects all discovered secrets.
///
/// This is a convenience wrapper around [`scan_all_files_for_keys_streaming`].
/// Binary files, oversized files (> 2 MiB), and files in [`EXCLUDED_DIRS`](crate::config::EXCLUDED_DIRS)
/// directories are skipped automatically.
///
/// # Arguments
///
/// * `path` - Root directory to start the recursive walk from.
pub fn scan_all_files_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_all_files_for_keys_streaming(path, |m| matches.push(m));
    matches
}

/// Scans all readable text files under `path`, emitting matches via a callback.
///
/// This is the broadest scanner: it walks the entire file tree, respecting the
/// directory exclusion list, and applies all registered patterns to every scannable file.
/// For `.env*` files encountered during the walk, matches are automatically flagged
/// as hardcoded. For all other files, the [`is_hardcoded_in_line`](crate::matcher::is_hardcoded_in_line)
/// heuristic determines the flag value.
///
/// # Arguments
///
/// * `path` - Root directory to start scanning from.
/// * `on_match` - Closure called once for every discovered [`KeyMatch`].
pub fn scan_all_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| is_scannable_file(p))
        .filter_map(|p| read_text_file(&p).map(|content| (p, content)))
        .for_each(|(p, content)| {
            let hardcoded_by_default = is_env_file(&p);
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            if let Some(cached) = cache.get_matches_with_hash(&p, &content_hash) {
                emit_cached_matches(&p, cached, &mut on_match);
                return;
            }

            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                &p,
                &content,
                &patterns_list,
                hardcoded_by_default,
                &mut |m, key_hash| {
                    cached_matches.push(CacheMatch {
                        provider: m.provider.clone(),
                        line_number: m.line_number,
                        key_masked: m.key.clone(),
                        hardcoded: m.hardcoded,
                        key_hash,
                    });
                    on_match(m);
                },
            );
            cache.store(&p, content_hash, cached_matches);
        });

    cache.save();
}

/// Scans source code files under `path`, excluding `.env*` files, via a streaming callback.
///
/// This scanner is designed to find hardcoded secrets in application source code
/// (`.rs`, `.py`, `.ts`, etc.) separately from environment configuration files.
/// All matches originating from this scanner start with `hardcoded = false` and are
/// then refined by the line-level heuristic.
///
/// Useful when you want to audit CI pipelines or source repositories independently
/// of their runtime configuration.
///
/// # Arguments
///
/// * `path` - Root directory to start scanning from.
/// * `on_match` - Closure called once for every discovered [`KeyMatch`].
pub fn scan_project_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| is_scannable_file(p) && !is_env_file(p))
        .filter_map(|p| read_text_file(&p).map(|content| (p, content)))
        .for_each(|(p, content)| {
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            if let Some(cached) = cache.get_matches_with_hash(&p, &content_hash) {
                emit_cached_matches(&p, cached, &mut on_match);
                return;
            }

            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                &p,
                &content,
                &patterns_list,
                false,
                &mut |m, key_hash| {
                    cached_matches.push(CacheMatch {
                        provider: m.provider.clone(),
                        line_number: m.line_number,
                        key_masked: m.key.clone(),
                        hardcoded: m.hardcoded,
                        key_hash,
                    });
                    on_match(m);
                },
            );
            cache.store(&p, content_hash, cached_matches);
        });

    cache.save();
}

/// Scans known IDE configuration directories under `path` for leaked secrets.
///
/// Many modern IDEs (VS Code, JetBrains, Antigravity) store workspace-scoped
/// configuration and AI completions caches in hidden directories under the project root.
/// These files are sometimes accidentally committed or left unprotected, making them
/// a non-obvious source of credential exposure.
///
/// This scanner walks only the directories listed in [`IDE_DIRS`] relative to `path`
/// and ignores any that do not exist. Binary and oversized files are skipped.
///
/// # Arguments
///
/// * `path` - Root directory of the workspace (not the IDE directory itself).
/// * `on_match` - Closure called once for every discovered [`KeyMatch`].
pub fn scan_ide_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    IDE_DIRS
        .iter()
        .map(|dir_name| root.join(dir_name))
        .filter(|ide_root| ide_root.is_dir())
        .flat_map(|ide_root| WalkDir::new(ide_root).into_iter().filter_map(|e| e.ok()))
        .map(|e| e.into_path())
        .filter(|p| is_scannable_file(p))
        .filter_map(|p| read_text_file(&p).map(|content| (p, content)))
        .for_each(|(p, content)| {
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            if let Some(cached) = cache.get_matches_with_hash(&p, &content_hash) {
                emit_cached_matches(&p, cached, &mut on_match);
                return;
            }

            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                &p,
                &content,
                &patterns_list,
                false,
                &mut |m, key_hash| {
                    cached_matches.push(CacheMatch {
                        provider: m.provider.clone(),
                        line_number: m.line_number,
                        key_masked: m.key.clone(),
                        hardcoded: m.hardcoded,
                        key_hash,
                    });
                    on_match(m);
                },
            );
            cache.store(&p, content_hash, cached_matches);
        });

    cache.save();
}

/// Reconstructs [`KeyMatch`] values from cached [`CacheMatch`] entries and emits them.
///
/// This is called when a file's content hash matches the cached one, allowing the scanner
/// to bypass actual regex processing and serve results directly from the index.
///
/// # Arguments
///
/// * `file_path` - Absolute path to the file whose cached matches are being re-emitted.
/// * `cached` - Slice of stored match metadata from the cache index.
/// * `on_match` - The caller's output closure.
fn emit_cached_matches<F>(file_path: &Path, cached: &[CacheMatch], on_match: &mut F)
where
    F: FnMut(KeyMatch),
{
    for m in cached {
        on_match(KeyMatch {
            file_path: file_path.to_path_buf(),
            line_number: m.line_number,
            provider: m.provider.clone(),
            key: m.key_masked.clone(),
            hardcoded: m.hardcoded,
        });
    }
}
