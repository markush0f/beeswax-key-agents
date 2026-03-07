pub mod config;
pub mod file_utils;
mod matcher;
pub mod patterns;
pub mod scan;
pub mod types;

pub use scan::{
    scan_all_files_for_keys, scan_all_files_for_keys_streaming, scan_env_for_keys,
    scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming,
    scan_project_files_for_keys_streaming,
};
pub use types::KeyMatch;
