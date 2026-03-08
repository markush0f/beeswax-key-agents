use regex::Regex;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: Regex,
    pub excluded_prefixes: &'static [&'static str],
}

impl SecretPattern {
    pub fn first_capture<'a>(&self, line: &'a str) -> Option<&'a str> {
        self.regex
            .captures_iter(line)
            .filter_map(|caps| caps.get(1).map(|matched| matched.as_str()))
            .find(|key| self.allows_key(key))
    }

    pub fn allows_key(&self, key: &str) -> bool {
        !self
            .excluded_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix))
    }
}

pub fn get_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "OpenRouter API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])(sk-or-v1-[0-9a-fA-F]{64})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:sk-proj-|sk-)[A-Za-z0-9_-]{32,})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            excluded_prefixes: &["sk-or-v1-"],
        },
        SecretPattern {
            name: "Deepseek API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(sk-[a-zA-Z0-9]{32})(?:$|[^A-Za-z0-9])").unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Gemini API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(AIza[0-9A-Za-z_-]{35})(?:$|[^A-Za-z0-9])")
                .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Grok API Key",
            regex: Regex::new(
                r#"(?:^|[^A-Za-z0-9_])XAI_API_KEY\s*[:=]\s*["']?([A-Za-z0-9._-]{24,})(?:["']|$)"#,
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Anthropic API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(sk-ant-[A-Za-z0-9_-]{20,})(?:$|[^A-Za-z0-9])")
                .unwrap(),
            excluded_prefixes: &[],
        },
        SecretPattern {
            name: "Ollama API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:ollama_[A-Za-z0-9_-]{20,}|sk-ollama-[A-Za-z0-9_-]{20,}|[0-9a-fA-F]{32}\.[A-Za-z0-9_-]{20,}))(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
            excluded_prefixes: &[],
        },
    ]
}
