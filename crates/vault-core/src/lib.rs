pub mod patterns;

use patterns::{get_patterns, SecretPattern};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct KeyMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub provider: String,
    pub key: String,
}

pub fn scan_env_for_keys(path: &str) -> Vec<KeyMatch> {
    let mut matches = Vec::new();
    let patterns_list = get_patterns();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();

        if !is_env_file(p) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(p) {
            find_matches_in_content(p, &content, &patterns_list, &mut matches);
        }
    }

    matches
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

fn find_matches_in_content(
    file_path: &Path,
    content: &str,
    patterns: &[SecretPattern],
    matches: &mut Vec<KeyMatch>,
) {
    let extracted_keys = content.lines().enumerate().flat_map(|(i, line)| {
        patterns.iter().flat_map(move |pattern| {
            pattern.regex.captures_iter(line).filter_map(move |caps| {
                caps.get(1).map(|matched| KeyMatch {
                    file_path: file_path.to_path_buf(),
                    line_number: i + 1,
                    provider: pattern.name.to_string(),
                    key: mask_key(matched.as_str()),
                })
            })
        })
    });

    matches.extend(extracted_keys);
}

fn mask_key(val: &str) -> String {
    if val.len() >= 12 {
        format!("{}...{}", &val[..10], &val[val.len() - 4..])
    } else {
        "****".to_string()
    }
}
