use regex::Regex;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct OpenAiKeyMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub key: String,
}

pub fn scan_env_for_openai_keys(path: &str) -> Vec<OpenAiKeyMatch> {
    let mut matches = Vec::new();

    // Regex para buscar claves tipo sk-proj-... o sk-...
    let key_regex = Regex::new(r"(sk-proj-[a-zA-Z0-9_\-]{20,}|sk-[a-zA-Z0-9_\-]{20,})").unwrap();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();

        if p.is_file() {
            if let Some(filename) = p.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(".env") {
                    if let Ok(content) = fs::read_to_string(p) {
                        for (i, line) in content.lines().enumerate() {
                            for caps in key_regex.captures_iter(line) {
                                if let Some(matched) = caps.get(1) {
                                    let val = matched.as_str();
                                    let masked = if val.len() >= 12 {
                                        format!("{}...{}", &val[..10], &val[val.len() - 4..])
                                    } else {
                                        "****".to_string()
                                    };

                                    matches.push(OpenAiKeyMatch {
                                        file_path: p.to_path_buf(),
                                        line_number: i + 1,
                                        key: masked,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    matches
}
