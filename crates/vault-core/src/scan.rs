use std::path::Path;

use walkdir::WalkDir;

use crate::config::IDE_DIRS;
use crate::file_utils::{is_env_file, is_ignored_dir, is_scannable_file, read_text_file};
use crate::matcher::find_matches_in_content_streaming;
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

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();

        if !is_env_file(p) {
            continue;
        }

        if let Some(content) = read_text_file(p) {
            find_matches_in_content_streaming(p, &content, &patterns_list, true, &mut on_match);
        }
    }
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

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let p = entry.path();

        if !is_scannable_file(p) {
            continue;
        }

        if let Some(content) = read_text_file(p) {
            let hardcoded_by_default = is_env_file(p);
            find_matches_in_content_streaming(
                p,
                &content,
                &patterns_list,
                hardcoded_by_default,
                &mut on_match,
            );
        }
    }
}

pub fn scan_project_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        let p = entry.path();

        if !is_scannable_file(p) || is_env_file(p) {
            continue;
        }

        if let Some(content) = read_text_file(p) {
            find_matches_in_content_streaming(p, &content, &patterns_list, false, &mut on_match);
        }
    }
}

pub fn scan_ide_files_for_keys_streaming<F>(path: &str, mut on_match: F)
where
    F: FnMut(KeyMatch),
{
    let patterns_list = get_patterns();
    let root = Path::new(path);

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

            if let Some(content) = read_text_file(p) {
                find_matches_in_content_streaming(
                    p,
                    &content,
                    &patterns_list,
                    false,
                    &mut on_match,
                );
            }
        }
    }
}
