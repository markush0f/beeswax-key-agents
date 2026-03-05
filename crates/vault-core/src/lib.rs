pub mod patterns;

use patterns::{SecretPattern, get_patterns};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct KeyMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub provider: String,
    pub key: String,
    pub hardcoded: bool,
}

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

    for dir_name in [".antigravity-server", ".vscode", ".idea"] {
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

fn is_env_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with(".env"))
        .unwrap_or(false)
}

fn is_scannable_file(path: &Path) -> bool {
    path.is_file()
}

fn is_ignored_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | ".idea" | ".vscode" | ".antigravity-server"
            )
        })
        .unwrap_or(false)
}

fn read_text_file(path: &Path) -> Option<String> {
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

fn find_matches_in_content_streaming<F>(
    file_path: &Path,
    content: &str,
    patterns: &[SecretPattern],
    hardcoded_by_default: bool,
    on_match: &mut F,
) where
    F: FnMut(KeyMatch),
{
    for (i, line) in content.lines().enumerate() {
        for pattern in patterns {
            for caps in pattern.regex.captures_iter(line) {
                if let Some(matched) = caps.get(1) {
                    let key = matched.as_str();

                    on_match(KeyMatch {
                        file_path: file_path.to_path_buf(),
                        line_number: i + 1,
                        provider: pattern.name.to_string(),
                        key: mask_key(key),
                        hardcoded: hardcoded_by_default || is_hardcoded_in_line(line, key),
                    });
                }
            }
        }
    }
}

fn is_hardcoded_in_line(line: &str, key: &str) -> bool {
    let quoted_double = format!("\"{key}\"");
    let quoted_single = format!("'{key}'");
    let quoted_backtick = format!("`{key}`");

    if line.contains(&quoted_double)
        || line.contains(&quoted_single)
        || line.contains(&quoted_backtick)
    {
        return true;
    }

    let trimmed = line.trim();
    trimmed.contains(key) && (trimmed.contains('=') || trimmed.contains(':'))
}

fn mask_key(val: &str) -> String {
    if val.len() >= 12 {
        format!("{}...{}", &val[..10], &val[val.len() - 4..])
    } else {
        "****".to_string()
    }
}
