use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub matches: Vec<KeyMatch>,
    pub files_scanned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMatch {
    pub file_path: String,
    pub line_number: usize,
    pub key_type: String,
    pub matched_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanType {
    Quick,
    Deep,
    Custom,
}
