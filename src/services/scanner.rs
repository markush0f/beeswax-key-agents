use crate::models::scan::{KeyMatch, ScanResult, ScanType};
use regex::Regex;
use std::fs;
use walkdir::WalkDir;

pub struct ScannerService;

impl ScannerService {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_path(&self, target_path: &str, _scan_type: ScanType) -> Result<ScanResult, String> {
        let mut matches = Vec::new();
        let mut files_scanned = 0;

        let key_regex = Regex::new(
            r"(?i)(api[_-]?key|secret|token|password)[\s]*[:=][\s]*['\x22]?([a-zA-Z0-9_\-]{16,})['\x22]?"
        ).unwrap();
        let aws_regex = Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap();

        let walker = WalkDir::new(target_path).into_iter().filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            if path.is_file() {
                files_scanned += 1;

                if let Ok(content) = fs::read_to_string(path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if let Some(caps) = key_regex.captures(line) {
                            if let Some(matched) = caps.get(2) {
                                let val = matched.as_str();
                                let masked = if val.len() >= 8 {
                                    format!("{}...{}", &val[..4], &val[val.len() - 4..])
                                } else {
                                    "****".to_string()
                                };
                                matches.push(KeyMatch {
                                    file_path: path.to_string_lossy().to_string(),
                                    line_number: line_num + 1,
                                    key_type: "Generic Key/Token".to_string(),
                                    matched_content: masked,
                                });
                            }
                        }
                        if let Some(caps) = aws_regex.captures(line) {
                            if let Some(matched) = caps.get(1) {
                                let val = matched.as_str();
                                matches.push(KeyMatch {
                                    file_path: path.to_string_lossy().to_string(),
                                    line_number: line_num + 1,
                                    key_type: "AWS Access Key".to_string(),
                                    // Normally you shouldn't mask AWS AKIAs completely, it's safe to see them.
                                    matched_content: val.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(ScanResult {
            matches,
            files_scanned,
        })
    }
}
