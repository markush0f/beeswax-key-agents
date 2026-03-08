use std::path::Path;

use crate::patterns::SecretPattern;
use crate::types::KeyMatch;

/// Scans a given text content for secrets line by line and invokes a callback for each match.
///
/// This function acts as the core matching engine. It leverages the provided `patterns`,
/// iterates over every line in `content`, and extracts matches. For every valid match,
/// the provided continuous callback `on_match` is called.
///
/// To optimize subsequent scans, a BLAKE3 hash of the secret is also computed and passed
/// to the callback; this replaces storing the raw key in cache files.
///
/// # Arguments
/// * `file_path` - The path of the file being scanned (used to populate the `KeyMatch`).
/// * `content` - The full string content to inspect.
/// * `patterns` - A slice of `SecretPattern` definitions to match against.
/// * `hardcoded_by_default` - If `true`, all matches are flagged as hardcoded (useful for `.env` files).
///   If `false`, a heuristic (`is_hardcoded_in_line`) decides if the secret is likely hardcoded.
/// * `on_match` - A mutable closure `FnMut(KeyMatch, String)` called for each discovered secret.
///   The closure receives the constructed `KeyMatch` and the BLAKE3 hash of the key.

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

/// Applies a best-effort heuristic to determine if a matched secret is hardcoded in source code.
///
/// This checks whether the line contains the secret surrounded by standard string literal
/// quotes (`"`, `'`, or `` ` ``) or if it exists within an assignment-like construct (`=` or `:`).
///
/// # Arguments
/// * `line` - The full line of text containing the match.
/// * `key` - The extracted secret key string.
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

/// Masks a sensitive key string for safe console output and reporting execution.
///
/// If the key is 12 characters or longer, it retains the first 10 characters and
/// the last 4 characters, separating them with `...` (e.g. `sk-proj-abc...1234`).
/// Keys shorter than 12 characters are fully redacted as `****`.
pub(crate) fn mask_key(val: &str) -> String {
    if val.len() >= 12 {
        format!("{}...{}", &val[..10], &val[val.len() - 4..])
    } else {
        "****".to_string()
    }
}
