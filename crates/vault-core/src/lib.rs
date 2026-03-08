//! # vault-core
//!
//! `vault-core` is the foundational library powering secret detection across local project trees.
//! It exposes a streaming scanning API that identifies exposed API keys and sensitive credentials
//! by walking file system trees and applying configurable regex pattern sets.
//!
//! ## Architecture Overview
//!
//! The crate is split into focused modules:
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`patterns`] | Defines regex-based [`SecretPattern`](patterns::SecretPattern) descriptors for each provider. |
//! | [`scan`] | High-level scanning entry points (env files, IDE dirs, full project trees). |
//! | [`matcher`] | Core regex matching engine with BLAKE3-based key hashing. |
//! | [`cache`] | Persistent scan index to skip unchanged files on repeated runs. |
//! | [`file_utils`] | File system predicates (env detection, binary file exclusion, ignore lists). |
//! | [`config`] | Compile-time constants for directory exclusion and IDE directory roots. |
//! | [`types`] | Shared data types used across the crate boundary (e.g., [`KeyMatch`]). |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use vault_core::scan_env_for_keys;
//!
//! let matches = scan_env_for_keys("/path/to/project");
//! for m in &matches {
//!     println!("[{}] Found {} key in {:?} at line {}", m.provider, m.key, m.file_path, m.line_number);
//! }
//! ```
//!
//! For large repositories, prefer the streaming variants to avoid materializing all matches in memory:
//!
//! ```rust,no_run
//! use vault_core::scan_env_for_keys_streaming;
//!
//! scan_env_for_keys_streaming("/path/to/project", |m| {
//!     println!("🔑 Secret found: {:?}", m);
//! });
//! ```
//!
//! ## Adding New Providers
//!
//! Extend [`patterns::get_patterns`] to register a new secret type. The match will immediately
//! be detected by all scanning functions — no other code changes required.

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
