use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct KeyMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub provider: String,
    pub key: String,
    pub hardcoded: bool,
}
