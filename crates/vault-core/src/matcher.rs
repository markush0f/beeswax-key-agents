//! Core regex matching engine for the secret detection pipeline.
//!
//! This module contains the lowest-level scanning routine: given a block of text
//! and a slice of [`SecretPattern`](crate::patterns::SecretPattern)s, it emits
//! every discovered secret via a streaming callback together with a BLAKE3 hash
//! of the raw value. Two supporting heuristics live here as well:
//!
//! - [`is_hardcoded_in_line`]: determines whether a match looks like a literal
//!   assignment rather than a runtime variable reference.
//! - [`mask_key`]: redacts all but the outer characters of a secret before it
//!   is stored or displayed.

use std::path::Path;

use crate::patterns::SecretPattern;
use crate::types::KeyMatch;

/// Scans a text string for secrets line by line and calls a callback for each match found.
///
/// This is the innermost and most performance-critical function in the scanning pipeline.
/// It does not perform any I/O — all file reading and caching logic is handled by
/// [`crate::scan`]. All `scan_*` functions ultimately delegate here.
///
/// ## Processing Order
///
/// For every line in `content`, every pattern in `patterns` is applied. If the pattern's
/// regex produces a capture group match and the key passes the pattern's exclusion list,
/// the callback is invoked with:
///
/// 1. A fully populated [`KeyMatch`] (with the key already masked).
/// 2. The BLAKE3 hex hash of the **raw** (unmasked) key, used for deduplication in the cache.
///
/// ## Hardcoded Detection
///
/// When `hardcoded_by_default` is `true` (e.g., when scanning `.env` files), every match
/// is flagged as hardcoded unconditionally. Otherwise, [`is_hardcoded_in_line`] is used
/// to determine whether the surrounding source context suggests a literal assignment.
///
/// # Arguments
///
/// * `file_path` - Source file path embedded into each emitted [`KeyMatch`].
/// * `content` - The full text content of the file, as a UTF-8 string slice.
/// * `patterns` - Slice of compiled [`SecretPattern`]s to apply.
/// * `hardcoded_by_default` - If `true`, all matches are unconditionally marked as hardcoded.
/// * `on_match` - Mutable closure called once per discovered secret.
///   Receives `(KeyMatch, key_hash: String)`.
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use vault_core::patterns::get_patterns;
///
/// // find_matches_in_content_streaming_with_hash is internal to vault-core.
/// // Access scanning functionality through the public scan::* entry points instead.
/// // See: vault_core::scan_env_for_keys, vault_core::scan_all_files_for_keys, etc.
/// ```
pub fn find_matches_in_content_streaming_with_hash<F>(
    file_path: &Path,
    content: &str,
    patterns: &[SecretPattern],
    hardcoded_by_default: bool,
    on_match: &mut F,
) where
    F: FnMut(KeyMatch, String),
{
    content
        .lines()
        .enumerate()
        .flat_map(|(i, line)| {
            patterns.iter().flat_map(move |pattern| {
                pattern.regex.captures_iter(line).filter_map(move |caps| {
                    let matched = caps.get(1)?;
                    let key = matched.as_str();

                    if !pattern.allows_key(key) {
                        return None;
                    }

                    Some((i, line, pattern, key.to_string()))
                })
            })
        })
        .for_each(|(i, line, pattern, key)| {
            let key_hash = blake3::hash(key.as_bytes()).to_hex().to_string();

            on_match(
                KeyMatch {
                    file_path: file_path.to_path_buf(),
                    line_number: i + 1,
                    provider: pattern.name.to_string(),
                    key: mask_key(&key),
                    hardcoded: hardcoded_by_default || is_hardcoded_in_line(line, &key),
                },
                key_hash,
            );
        });
}

/// Applies a heuristic to determine whether a secret appears to be hardcoded in source code.
///
/// The heuristic considers a key hardcoded if it satisfies **either** of two conditions:
///
/// 1. **Quoted literal**: The key appears surrounded by `"..."`, `'...'`, or `` `...` ``.
///    This covers most programming languages' string literal syntax.
///
/// 2. **Assignment context**: The trimmed line contains the key **and** includes an `=`
///    or `:` character, suggesting a variable assignment or config entry (e.g., `KEY=value`
///    or `key: value`).
///
/// # Limitations
///
/// This is a best-effort heuristic and can produce both false positives (e.g., a key
/// incorrectly inferred as hardcoded from a log line) and false negatives (e.g., a
/// multi-line string literal). It is intentionally simple to stay fast at scale.
///
/// # Arguments
///
/// * `line` - The full source line containing the match.
/// * `key` - The raw (unmasked) secret key string.
pub(crate) fn is_hardcoded_in_line(line: &str, key: &str) -> bool {
    let quoted_double = format!("\"{key}\"");
    let quoted_single = format!("'{key}'");
    let quoted_backtick = format!("`{key}`");

    if line.contains(&quoted_double)
        || line.contains(&quoted_single)
        || line.contains(&quoted_backtick)
    {
        return true;
    }

    let trimmed = line.trim();
    trimmed.contains(key) && (trimmed.contains('=') || trimmed.contains(':'))
}

/// Masks a sensitive key string for safe display and logging.
///
/// The masking strategy preserves just enough of the key to be recognizable
/// while preventing accidental secret exposure in logs, UIs, or cache files:
///
/// - Keys **≥ 12 characters**: retain the first 10 and last 4 characters,
///   separated by `...` (e.g., `sk-proj-12...abcd`).
/// - Keys **< 12 characters**: fully replaced with `****` to avoid leaking
///   short keys where any visible fragment would reveal too much.
///
/// # Arguments
///
/// * `val` - The raw secret key string to mask.
///
/// # Examples
///
/// ```rust
/// // Long key: retain extremities
/// // "sk-proj-1234567890abcd" → "sk-proj-12...abcd"
///
/// // Short key: fully redacted
/// // "secret" → "****"
/// ```
pub(crate) fn mask_key(val: &str) -> String {
    if val.len() >= 12 {
        format!("{}...{}", &val[..10], &val[val.len() - 4..])
    } else {
        "****".to_string()
    }
}
