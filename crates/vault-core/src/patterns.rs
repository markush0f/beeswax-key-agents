//! Secret pattern registry and [`SecretPattern`] descriptor type.
//!
//! This module is the single source of truth for all supported secret providers.
//! To add support for a new API key format, implement a new [`SecretPattern`] entry
//! in [`get_patterns`] — no other code change is required. The CLI, the TUI, and all
//! scanning functions automatically discover new patterns at runtime.
//!
//! ## Regex Requirements
//!
//! Every pattern's regular expression **must** wrap the target secret in the **first
//! capture group** (`(...)`). The rest of the expression typically anchors or provides
//! non-capturing context to reduce false positives (e.g., requiring a non-alphanumeric
//! boundary on each side of the key).
//!
//! ## Exclusion Prefixes
//!
//! Some provider key formats are supersets of others (e.g., OpenRouter keys start with
//! `sk-or-v1-`, which also matches OpenAI's `sk-` pattern). The `excluded_prefixes`
//! field allows a pattern to explicitly reject keys that belong to a sibling pattern,
//! preventing double-counting or misclassification.

use regex::Regex;

/// Descriptor for a detectable secret type.
///
/// A `SecretPattern` bundles everything the scanner and UI need to identify,
/// display, and style a class of secrets. Instances are constructed in [`get_patterns`]
/// and are typically used as a read-only slice throughout a scan run.
///
/// # Thread Safety
///
/// `SecretPattern` references only `'static` string slices and compiled [`Regex`] objects,
/// making it safe to share across threads after construction.
pub struct SecretPattern {
    /// Human-readable name of the secret provider (e.g., `"OpenAI API Key"`).
    ///
    /// Used in [`crate::types::KeyMatch::provider`] and displayed in the TUI match list.
    pub name: &'static str,

    /// Shortened label for compact UI elements such as bar charts.
    ///
    /// Should be ≤ 8 characters to fit within chart column widths (e.g., `"OpenAI"`, `"Anthro"`).
    pub short_name: &'static str,

    /// RGB color assigned to this provider in the TUI.
    ///
    /// Stored as `(red, green, blue)` where each component is in `[0, 255]`.
    /// Choosing distinct, perceptually separable colors improves readability of
    /// the provider bar chart.
    pub color: (u8, u8, u8),

    /// Compiled regular expression that matches the secret.
    ///
    /// The secret value **must** be captured in the **first capture group**. Surrounding
    /// context (word boundaries, non-alphanumeric guards) should use non-capturing groups.
    pub regex: Regex,

    /// Key prefixes that disqualify an otherwise-matching key.
    ///
    /// If a captured key string starts with any entry in this slice, the match is
    /// discarded. Use this to resolve ambiguity between patterns that share prefixes
    /// (e.g., excluding `"sk-or-v1-"` from the OpenAI pattern).
    pub excluded_prefixes: &'static [&'static str],
}

impl SecretPattern {
    /// Searches a line of text for the first valid match of this pattern.
    ///
    /// Iterates over all regex captures in `line`, extracting the first capture group
    /// from each. Each candidate is validated against [`excluded_prefixes`](Self::excluded_prefixes)
    /// before being returned.
    ///
    /// # Arguments
    ///
    /// * `line` - A string slice containing the text to inspect.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` — the raw secret string if a valid match is found.
    /// * `None` — if no match is found, or all matches are filtered by the exclusion list.
    pub fn first_capture<'a>(&self, line: &'a str) -> Option<&'a str> {
        self.regex
            .captures_iter(line)
            .filter_map(|caps| caps.get(1).map(|matched| matched.as_str()))
            .find(|key| self.allows_key(key))
    }

    /// Returns `true` if the given key is **not** rejected by any exclusion prefix.
    ///
    /// A key is rejected when it starts with any entry in [`excluded_prefixes`](Self::excluded_prefixes).
    /// This is called internally by [`first_capture`](Self::first_capture) and by
    /// [`crate::matcher::find_matches_in_content_streaming_with_hash`].
    ///
    /// # Arguments
    ///
    /// * `key` - The raw candidate secret string.
    pub fn allows_key(&self, key: &str) -> bool {
        !self
            .excluded_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix))
    }
}

/// Returns the complete list of all registered secret patterns.
///
/// This function is the single authoritative source for every provider supported by
/// `vault-core`. It is called once at the beginning of each scan run and the resulting
/// `Vec` is passed down through the entire scanning pipeline.
///
/// ## Adding a New Provider
///
/// Append a new [`SecretPattern`] to the returned `Vec`. Choose:
///
/// 1. A descriptive `name` and a short `short_name` (≤ 8 chars).
/// 2. A distinctive RGB `color` that doesn't clash visually with existing providers.
/// 3. A regex where the **first capture group** extracts the secret value precisely.
/// 4. Any `excluded_prefixes` needed to avoid overlap with other patterns.
///
/// ## Currently Supported Providers
///
/// | Provider | Prefix / Format |
/// |---|---|
/// | OpenRouter | `sk-or-v1-` + 64 hex chars |
/// | OpenAI | `sk-proj-` or `sk-` + 32+ alphanumeric chars |
/// | Deepseek | `sk-` + 32 alphanumeric chars (exact) |
/// | Gemini | `AIza` + 35 base64url chars |
/// | Grok (xAI) | `XAI_API_KEY=` env var assignment |
/// | Anthropic | `sk-ant-` + 20+ alphanumeric chars |
/// | Ollama | `ollama_` / `sk-ollama-` prefixed or hex token format |
pub fn get_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "OpenRouter API Key",
            short_name: "OpenRtr",
            color: (0, 255, 255), // Cyan
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])(sk-or-v1-[0-9a-fA-F]{64})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "OpenAI API Key",
            short_name: "OpenAI",
            color: (0, 255, 0), // Green
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:sk-proj-|sk-)[A-Za-z0-9_-]{32,})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            // Prevents OpenRouter keys (sk-or-v1-...) from matching this pattern.
            excluded_prefixes: &["sk-or-v1-"],
        },
        SecretPattern {
            name: "Deepseek API Key",
            short_name: "DpSk",
            color: (255, 255, 0), // Yellow
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(sk-[a-zA-Z0-9]{32})(?:$|[^A-Za-z0-9])").unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Gemini API Key",
            short_name: "Gemini",
            color: (0, 0, 255), // Blue
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(AIza[0-9A-Za-z_-]{35})(?:$|[^A-Za-z0-9])")
                .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Grok API Key",
            short_name: "Grok",
            color: (255, 0, 255), // Magenta
            regex: Regex::new(
                r#"(?:^|[^A-Za-z0-9_])XAI_API_KEY\s*[:=]\s*["']?([A-Za-z0-9._-]{24,})(?:["']|$)"#,
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Anthropic API Key",
            short_name: "Anthro",
            color: (255, 165, 0), // Orange
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(sk-ant-[A-Za-z0-9_-]{20,})(?:$|[^A-Za-z0-9])")
                .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Ollama API Key",
            short_name: "Ollama",
            color: (255, 100, 100), // LightRed
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:ollama_[A-Za-z0-9_-]{20,}|sk-ollama-[A-Za-z0-9_-]{20,}|[0-9a-fA-F]{32}\.[A-Za-z0-9_-]{20,}))(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
    ]
}
