use regex::Regex;

/// Defines a signature for a detectable secret (e.g., an API key).
///
/// A pattern relies on a regular expression to identify the secret, but can also
/// specify prefixes that invalidate the match to prevent false positives (e.g.,
/// avoiding overlap between an OpenAI and OpenRouter key).
pub struct SecretPattern {
    /// Human-readable name of the secret provider (e.g. "OpenAI API Key").
    pub name: &'static str,
    /// Short label for tight UI spaces like BarCharts (e.g. "OpenAI", "Anthro").
    pub short_name: &'static str,
    /// RGB color tuple used by the UI representation (r, g, b).
    pub color: (u8, u8, u8),
    /// The compiled regular expression that matches the secret. To be extracted,
    /// the secret must be exactly matched by the first capture group.
    pub regex: Regex,
    /// A list of string prefixes. If the captured secret begins with any of these,
    /// it will be discarded as a false positive.
    pub excluded_prefixes: &'static [&'static str],
}

impl SecretPattern {
    /// Attempts to find a matching secret in the given line of text.
    ///
    /// This method iterates over all regex captures in the line. For each capture,
    /// it extracts the first capture group (which must contain the exact secret string).
    /// It then verifies the extracted secret against the `excluded_prefixes` list.
    ///
    /// # Arguments
    /// * `line` - A string slice containing the text to inspect.
    ///
    /// # Returns
    /// * `Some(&str)` containing the matched secret if found and allowed.
    /// * `None` if no match is found, or if all matches are filtered out by excluded prefixes.
    pub fn first_capture<'a>(&self, line: &'a str) -> Option<&'a str> {
        self.regex
            .captures_iter(line)
            .filter_map(|caps| caps.get(1).map(|matched| matched.as_str()))
            .find(|key| self.allows_key(key))
    }

    /// Verifies if a matched key is allowed based on the pattern's exclusion list.
    ///
    /// Returns `true` if the key does not start with any of the `excluded_prefixes`.
    pub fn allows_key(&self, key: &str) -> bool {
        !self
            .excluded_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix))
    }
}

/// Returns a pre-configured vector of all supported `SecretPattern` detectors.
///
/// This list currently includes detectors for:
/// * OpenRouter
/// * OpenAI
/// * Deepseek
/// * Gemini
/// * Grok
/// * Anthropic
/// * Ollama
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
