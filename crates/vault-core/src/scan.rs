use std::path::Path;

use walkdir::WalkDir;

use crate::cache::{Cache, CacheMatch};
use crate::config::IDE_DIRS;
use crate::file_utils::{is_env_file, is_ignored_dir, is_scannable_file, read_text_file};
use crate::matcher::find_matches_in_content_streaming_with_hash;
use crate::patterns::get_patterns;
use crate::types::KeyMatch;

/// Recursively scans a path exclusively for environment variables files (e.g., `.env`) and returns all discovered secrets.
///
/// This is a convenience wrapper around [`scan_env_for_keys_streaming`] that collects all matches
/// into a `Vec`. For large repositories, consider using the streaming variant to process matches
/// as they are found and avoid allocating memory for all of them at once.
pub fn scan_env_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_env_for_keys_streaming(path, |m| matches.push(m));
    matches
}

/// Recursively scans a path for `.env*` files, streaming matches via a callback.
///
/// All secrets found in these files are automatically considered "hardcoded" because
/// they are being plainly assigned in a configuration wrapper.
///
/// # Arguments
/// * `path` - The root directory or file path to start scanning from.
/// * `on_match` - A mutable closure that is called immediately when a leak is discovered.
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

/// Recursively scans all scannable files in a given path and returns all discovered secrets.
///
/// This is a convenience wrapper around [`scan_all_files_for_keys_streaming`] that collects all matches
/// into a `Vec`. This function will ignore files that are not considered scannable (like binaries)
/// and respects ignores (like `.git/` or `node_modules/`).
pub fn scan_all_files_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_all_files_for_keys_streaming(path, |m| matches.push(m));
    matches
}

/// Recursively scans all scannable files in a given path, streaming matches via a callback.
///
/// # Arguments
/// * `path` - The root directory or file path to start scanning from.
/// * `on_match` - A mutable closure that is called immediately when a leak is discovered.
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

/// Recursively scans project source code for potential secrets, streaming matches via a callback.
///
/// This function acts like `scan_all_files_for_keys_streaming`, but strictly **excludes** `.env*`
/// environment files from the scope. It is useful when trying to verify if application code (e.g.,
/// Python, Typescript, Rust files) contains hardcoded credentials independently of configuration files.
/// All secrets matched here are assumed not to be hardcoded by default, relying instead on
/// heuristic evaluation line-by-line.
///
/// # Arguments
/// * `path` - The root directory or file path to start scanning from.
/// * `on_match` - A mutable closure that is called immediately when a leak is discovered.
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

/// Recursively scans known IDE configuration directories for potential leaked secrets.
///
/// Many IDEs (like VSCode, JetBrains suites) store workspace configuration in hidden
/// `.vscode` or `.idea` folders which may be unintentionally committed or tracked.
/// This scanner specifically targets those folders relative to the given `path`.
///
/// # Arguments
/// * `path` - The root directory of the workspace where the IDE folder resides.
/// * `on_match` - A mutable closure that is called immediately when a leak is discovered.
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
