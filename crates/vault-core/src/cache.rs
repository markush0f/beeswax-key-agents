use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMatch {
    pub provider: String,
    pub line_number: usize,
    pub key_masked: String,
    pub hardcoded: bool,
    pub key_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    mtime_secs: u64,
    size_bytes: u64,
    content_hash: String,
    matches: Vec<CacheMatch>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheIndex {
    version: u32,
    entries: HashMap<String, CacheEntry>,
}

pub struct Cache {
    root: PathBuf,
    index: CacheIndex,
    dirty: bool,
}

impl Cache {
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

    pub fn get_matches(&self, path: &Path) -> Option<&[CacheMatch]> {
        let key = path_key(&self.root, path);
        let entry = self.index.entries.get(&key)?;
        let (mtime_secs, size_bytes) = file_signature(path)?;

        if entry.mtime_secs == mtime_secs && entry.size_bytes == size_bytes {
            return Some(entry.matches.as_slice());
        }

        None
    }

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

fn cache_dir(root: &Path) -> PathBuf {
    root.join(".vault-cache")
}

fn cache_path(root: &Path) -> PathBuf {
    cache_dir(root).join("index.json")
}

fn path_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn file_signature(path: &Path) -> Option<(u64, u64)> {
    let meta = fs::metadata(path).ok()?;
    let size_bytes = meta.len();
    let mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or_else(|| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now
        });

    Some((mtime_secs, size_bytes))
}
