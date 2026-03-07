use std::path::Path;

use walkdir::WalkDir;

use crate::cache::{Cache, CacheMatch};
use crate::config::IDE_DIRS;
use crate::file_utils::{is_env_file, is_ignored_dir, is_scannable_file, read_text_file};
use crate::matcher::find_matches_in_content_streaming_with_hash;
use crate::patterns::get_patterns;
use crate::types::KeyMatch;

pub fn scan_env_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_env_for_keys_streaming(path, |m| matches.push(m));
    matches
}

pub fn scan_env_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();

        if !is_env_file(p) {
            continue;
        }

        if let Some(cached) = cache.get_matches(p) {
            emit_cached_matches(p, cached, &mut on_match);
            continue;
        }

        if let Some(content) = read_text_file(p) {
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                p,
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
            cache.store(p, content_hash, cached_matches);
        }
    }

    cache.save();
}

pub fn scan_all_files_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    scan_all_files_for_keys_streaming(path, |m| matches.push(m));
    matches
}

pub fn scan_all_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let p = entry.path();

        if !is_scannable_file(p) {
            continue;
        }

        if let Some(cached) = cache.get_matches(p) {
            emit_cached_matches(p, cached, &mut on_match);
            continue;
        }

        if let Some(content) = read_text_file(p) {
            let hardcoded_by_default = is_env_file(p);
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                p,
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
            cache.store(p, content_hash, cached_matches);
        }
    }

    cache.save();
}

pub fn scan_project_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let p = entry.path();

        if !is_scannable_file(p) || is_env_file(p) {
            continue;
        }

        if let Some(cached) = cache.get_matches(p) {
            emit_cached_matches(p, cached, &mut on_match);
            continue;
        }

        if let Some(content) = read_text_file(p) {
            let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            let mut cached_matches = Vec::new();
            find_matches_in_content_streaming_with_hash(
                p,
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
            cache.store(p, content_hash, cached_matches);
        }
    }

    cache.save();
}

pub fn scan_ide_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);
    let mut cache = Cache::load(root);

    for dir_name in IDE_DIRS {
        let ide_root = root.join(dir_name);
        if !ide_root.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&ide_root).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if !is_scannable_file(p) {
                continue;
            }

            if let Some(cached) = cache.get_matches(p) {
                emit_cached_matches(p, cached, &mut on_match);
                continue;
            }

            if let Some(content) = read_text_file(p) {
                let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
                let mut cached_matches = Vec::new();
                find_matches_in_content_streaming_with_hash(
                    p,
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
                cache.store(p, content_hash, cached_matches);
            }
        }
    }

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
