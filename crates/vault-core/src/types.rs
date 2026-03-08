use std::path::PathBuf;

/// Represents a single detected secret found during a file scan.
///
/// `KeyMatch` is the primary output type of all scanning functions in this crate.
/// It carries enough context to locate the secret in the source tree, understand
/// which provider it belongs to, and determine whether it is likely hardcoded.
///
/// # Key Masking
///
/// The `key` field is **never** the raw secret string. The scanner always masks
/// it (e.g., `sk-proj-ab...1234`) before populating this struct, making it
/// safe to log, display in a TUI, or send to an analytics backend.
///
/// # Example
///
/// ```rust
/// use vault_core::KeyMatch;
/// use std::path::PathBuf;
///
/// let m = KeyMatch {
///     file_path: PathBuf::from("/home/user/project/.env"),
///     line_number: 12,
///     provider: "OpenAI API Key".to_string(),
///     key: "sk-proj-ab...cdef".to_string(),
///     hardcoded: true,
/// };
///
/// println!("[{}] {} at line {}", m.provider, m.key, m.line_number);
/// ```
#[derive(Debug, Clone)]
pub struct KeyMatch {
    /// Absolute path to the file where the secret was found.
    pub file_path: PathBuf,
    /// 1-indexed line number within the file where the secret was matched.
    pub line_number: usize,
    /// Human-readable name of the secret provider (e.g., `"OpenAI API Key"`).
    pub provider: String,
    /// The masked representation of the discovered key (e.g., `"sk-proj-ab...cdef"`).
    /// This is intentionally truncated to avoid exposing sensitive values.
    pub key: String,
    /// Indicates whether the secret appears to be hardcoded in source code rather
    /// than sourced from an environment variable at runtime.
    ///
    /// This flag is set via heuristic analysis (quote/assignment detection) or
    /// unconditionally for `.env*` files.
    pub hardcoded: bool,
}
