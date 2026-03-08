//! `vault-core` provides the foundational logic for detecting leaked API keys and secrets.
//!
//! This crate contains the main scanning engine, which parses configurations,
//! iterates over directories, and uses pre-defined regex rules to detect
//! known formats for secrets like OpenAI, Anthropic, Gemini, and Deepseek keys.
//!
//! # Modules
//! * `patterns`: Defines the regex patterns for secret matching.
//! * `scan`: Contains the high-level iterators and streaming API to scan files.
//! * `matcher`: Implements the core regex matching and hashing operations.
//! * `cache`: Handles state persistence to avoid rescanning unchanged files.

mod cache;
pub mod config;
mod file_utils;
mod matcher;
pub mod patterns;
pub mod scan;
pub mod types;

#[cfg(test)]
mod tests;

pub use scan::{
    scan_all_files_for_keys, scan_all_files_for_keys_streaming, scan_env_for_keys,
    scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming,
    scan_project_files_for_keys_streaming,
};
pub use types::KeyMatch;
