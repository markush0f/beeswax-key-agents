//! Shared UI utility functions used across multiple submodules.
//!
//! This module keeps small, reusable helpers that don't belong to any specific
//! UI panel. Currently provides:
//!
//! - [`spinner_ascii`]: A 4-frame ASCII spinner driven by the global tick counter.
//! - [`elide_middle`]: A Unicode-aware string truncator that preserves both ends.

/// Returns a single-character ASCII spinner frame for the given animation tick.
///
/// Cycles through `["-", "\\", "|", "/"]` in order, producing a classic terminal
/// spinner animation when called once per tick. The tick is typically incremented
/// every 125 ms by the main event loop.
///
/// # Arguments
///
/// * `tick` - The current monotonic frame counter (wrapping `u64`).
///
/// # Examples
///
/// ```rust
///  Calling with consecutive ticks produces a rotating animation:
///  tick 0 → "-"
///  tick 1 → "\"
///  tick 2 → "|"
///  tick 3 → "/"
///  tick 4 → "-"  (wraps back)
/// ```
pub fn spinner_ascii(tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[(tick as usize) % FRAMES.len()]
}

/// Truncates a string to at most `max_len` Unicode scalar values, preserving both ends.
///
/// When the input exceeds `max_len`, the middle of the string is replaced with `"..."`.
/// This is more useful than right-truncation for long file paths, because the beginning
/// (drive / root) and the end (filename) are typically both meaningful to the user.
///
/// # Behaviour
///
/// - If `input.chars().count() <= max_len`, the original string is returned unchanged.
/// - If `max_len <= 3`, the result is `max_len` dots (e.g., `"..."`, `".."`, `"."`).
/// - Otherwise, the head and tail are split as evenly as possible around the `"..."` separator.
///
/// # Arguments
///
/// * `input` - The string to elide. May contain multibyte UTF-8 characters.
/// * `max_len` - Maximum number of Unicode scalar values in the output.
///
/// # Examples
///
/// ```rust
/// // "/home/user/very/long/path/to/file.rs" with max_len=20
/// // → "/home/user...file.rs"
/// ```
pub fn elide_middle(input: &str, max_len: usize) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    if len <= max_len {
        return input.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }

    let head = (max_len - 3) / 2;
    let tail = (max_len - 3) - head;
    let start: String = chars[..head].iter().collect();
    let end: String = chars[len - tail..].iter().collect();
    format!("{start}...{end}")
}
