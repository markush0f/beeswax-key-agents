//! Persistent scan cache to avoid reprocessing unchanged files.
//!
//! On repeated runs over the same project tree, rescanning every file from scratch
//! would be wasteful. This module implements a lightweight file-level cache stored
//! under `.vault-cache/index.json` at the scan root.
//!
//! ## How It Works
//!
//! Before scanning a file, the caller computes its BLAKE3 content hash. If the cache
//! already contains an entry for that file **with the same hash**, the cached matches
//! are returned immediately without reading the file again.
//!
//! After a scan, any new or changed entries are written back atomically using a
//! temp-file rename to avoid leaving a corrupt index on a crash or power failure.
//!
//! ## Cache Invalidation
//!
//! Invalidation is hash-based: if the content hash changes, the entry is evicted and
//! the file is rescanned. File size and mtime are stored for potential future use but
//! are not currently used as primary cache keys.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A single cached match result, mirroring the fields of [`crate::types::KeyMatch`]
/// minus the resolved file path (which is reconstructed at read time).
///
/// Storing the BLAKE3 `key_hash` instead of the raw key keeps the cache safe to
/// commit alongside source code if needed, since the hash cannot be reversed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMatch {
    /// Human-readable provider name (e.g., `"OpenAI API Key"`).
    pub provider: String,
    /// Line number where the match was found (1-indexed).
    pub line_number: usize,
    /// Masked key string (e.g., `"sk-proj-ab...cdef"`).
    pub key_masked: String,
    /// Whether the match was flagged as a hardcoded secret.
    pub hardcoded: bool,
    /// BLAKE3 hex digest of the raw secret value. Used for deduplication.
    pub key_hash: String,
}

/// Internal representation of a single cached file entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Last-modified timestamp in seconds since Unix epoch.
    mtime_secs: u64,
    /// File size in bytes at the time of the last scan.
    size_bytes: u64,
    /// BLAKE3 hex digest of the file's full content.
    content_hash: String,
    /// All secrets found the last time this file was scanned.
    matches: Vec<CacheMatch>,
}

/// Root structure serialised to `.vault-cache/index.json`.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheIndex {
    /// Schema version, reserved for future backwards-compatibility handling.
    version: u32,
    /// Map from relative file path string to the corresponding cache entry.
    entries: HashMap<String, CacheEntry>,
}

/// The runtime cache handle. Holds the deserialized index and tracks whether
/// any mutations have occurred to decide whether a `save()` call needs I/O.
pub struct Cache {
    /// Absolute path to the directory being scanned (used to relativize paths).
    root: PathBuf,
    /// The deserialized index loaded from disk (or an empty default).
    index: CacheIndex,
    /// Set to `true` whenever an entry is inserted via [`Cache::store`].
    dirty: bool,
}

impl Cache {
    /// Loads the cache index from `.vault-cache/index.json` relative to `root`.
    ///
    /// If the file does not exist, is unreadable, or fails JSON deserialization,
    /// an empty default index is returned. This makes the first run on a new
    /// project entirely transparent.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the scan. All file paths are stored
    ///   relative to this directory.
    pub fn load(root: &Path) -> Self {
        let path = cache_path(root);
        let index = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<CacheIndex>(&raw).ok())
            .unwrap_or_default();

        Self {
            root: root.to_path_buf(),
            index,
            dirty: false,
        }
    }

    /// Looks up cached matches for a file, returning them only if the file's
    /// content hash matches the stored one.
    ///
    /// # Arguments
    ///
    /// * `path` - Absolute path to the file being checked.
    /// * `content_hash` - The current BLAKE3 hash of the file's contents.
    ///
    /// # Returns
    ///
    /// * `Some(&[CacheMatch])` if the file is cached and its hash is unchanged.
    /// * `None` if the file is unknown or has been modified since last scan.
    pub fn get_matches_with_hash(&self, path: &Path, content_hash: &str) -> Option<&[CacheMatch]> {
        let key = path_key(&self.root, path);
        let entry = self.index.entries.get(&key)?;

        if entry.content_hash == content_hash {
            return Some(entry.matches.as_slice());
        }

        None
    }

    /// Inserts or replaces the cache entry for a file after it has been scanned.
    ///
    /// Marks the index as dirty so that [`Cache::save`] knows to flush to disk.
    ///
    /// # Arguments
    ///
    /// * `path` - Absolute path to the file that was scanned.
    /// * `content_hash` - BLAKE3 content hash at the time of the scan.
    /// * `matches` - All secrets found during this scan run.
    pub fn store(&mut self, path: &Path, content_hash: String, matches: Vec<CacheMatch>) {
        let key = path_key(&self.root, path);
        let (mtime_secs, size_bytes) = file_signature(path).unwrap_or((0, 0));

        self.index.entries.insert(
            key,
            CacheEntry {
                mtime_secs,
                size_bytes,
                content_hash,
                matches,
            },
        );
        self.dirty = true;
    }

    /// Flushes the in-memory index to `.vault-cache/index.json` if the cache was modified.
    ///
    /// The write is atomic: the JSON is first written to a `.json.tmp` sibling file
    /// and then renamed into place. This prevents a partially-written index from
    /// corrupting future runs if the process terminates mid-write.
    ///
    /// If the cache has not been modified since it was loaded, this is a no-op.
    pub fn save(&self) {
        if !self.dirty {
            return;
        }

        let dir = cache_dir(&self.root);
        let path = cache_path(&self.root);
        let _ = fs::create_dir_all(&dir);

        let Ok(json) = serde_json::to_string(&self.index) else {
            return;
        };

        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(tmp, path);
        }
    }
}

/// Returns the path to the cache directory (`.vault-cache/`) for a given root.
fn cache_dir(root: &Path) -> PathBuf {
    root.join(".vault-cache")
}

/// Returns the full path to the cache index file.
fn cache_path(root: &Path) -> PathBuf {
    cache_dir(root).join("index.json")
}

/// Converts an absolute file path to a root-relative string key for the index map.
///
/// Falls back to the full path string if the path cannot be stripped of the prefix
/// (which should not happen under normal operation).
fn path_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Returns `(mtime_secs, size_bytes)` for a given file path.
///
/// Both values fall back to `0` if the file metadata is unavailable. The mtime
/// is expressed as seconds since the Unix epoch.
fn file_signature(path: &Path) -> Option<(u64, u64)> {
    let meta = fs::metadata(path).ok()?;
    let size_bytes = meta.len();
    let mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        });

    Some((mtime_secs, size_bytes))
}
