use regex::Regex;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: Regex,
}

pub fn get_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "OpenRouter API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])(sk-or-v1-[0-9a-fA-F]{64})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
        },
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:sk-proj-|sk-(?!or-v1-))[A-Za-z0-9_-]{32,})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
        },
        SecretPattern {
            name: "Gemini API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(AIza[0-9A-Za-z_-]{35})(?:$|[^A-Za-z0-9])")
                .unwrap(),
        },
        SecretPattern {
            name: "Anthropic API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(sk-ant-[A-Za-z0-9_-]{20,})(?:$|[^A-Za-z0-9])")
                .unwrap(),
        },
        SecretPattern {
            name: "Ollama API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:ollama_[A-Za-z0-9_-]{20,}|sk-ollama-[A-Za-z0-9_-]{20,}|[0-9a-fA-F]{32}\.[A-Za-z0-9_-]{20,}))(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::get_patterns;

    #[test]
    fn matches_openrouter_keys() {
        let patterns = get_patterns();
        let openrouter = patterns
            .iter()
            .find(|p| p.name == "OpenRouter API Key")
            .expect("openrouter pattern should exist");

        let line = r#"OPENROUTER_API_KEY="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""#;
        let caps = openrouter
            .regex
            .captures(line)
            .expect("expected openrouter key match");
        assert!(caps.get(1).is_some());
    }

    #[test]
    fn does_not_classify_openrouter_as_openai() {
        let patterns = get_patterns();
        let openai = patterns
            .iter()
            .find(|p| p.name == "OpenAI API Key")
            .expect("openai pattern should exist");

        let line = r#"OPENROUTER_API_KEY="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""#;
        assert!(openai.regex.captures(line).is_none());
    }

    #[test]
    fn matches_anthropic_keys() {
        let patterns = get_patterns();
        let anthropic = patterns
            .iter()
            .find(|p| p.name == "Anthropic API Key")
            .expect("anthropic pattern should exist");

        let line = r#"ANTHROPIC_API_KEY="sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWX1234567890abcdEFGH""#;
        let caps = anthropic
            .regex
            .captures(line)
            .expect("expected anthropic key match");
        assert!(caps.get(1).is_some());
    }

    #[test]
    fn does_not_match_non_key_tokens() {
        let patterns = get_patterns();
        let anthropic = patterns
            .iter()
            .find(|p| p.name == "Anthropic API Key")
            .expect("anthropic pattern should exist");

        let line = "'asterisk-exception': {'id': 'Asterisk-exception'}";
        assert!(anthropic.regex.captures(line).is_none());
    }

    #[test]
    fn matches_ollama_keys() {
        let patterns = get_patterns();
        let ollama = patterns
            .iter()
            .find(|p| p.name == "Ollama API Key")
            .expect("ollama pattern should exist");

        let line = r#"OLLAMA_API_KEY="ollama_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456""#;
        let caps = ollama
            .regex
            .captures(line)
            .expect("expected ollama key match");
        assert!(caps.get(1).is_some());
    }

    #[test]
    fn matches_ollama_native_token_format() {
        let patterns = get_patterns();
        let ollama = patterns
            .iter()
            .find(|p| p.name == "Ollama API Key")
            .expect("ollama pattern should exist");

        let line = r#"OLLAMA_API_KEY="0972b6f6eb88495aa1f9f581189104f1._VH6UlaBHFRMsQ0vj-sRZYDq""#;
        let caps = ollama
            .regex
            .captures(line)
            .expect("expected native ollama token format match");
        assert!(caps.get(1).is_some());
    }

    #[test]
    fn does_not_match_generic_ollama_text() {
        let patterns = get_patterns();
        let ollama = patterns
            .iter()
            .find(|p| p.name == "Ollama API Key")
            .expect("ollama pattern should exist");

        let line = "Use ollama serve to run local models";
        assert!(ollama.regex.captures(line).is_none());
    }
}
