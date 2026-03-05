use regex::Regex;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: Regex,
}

pub fn get_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(
                r"(?:^|[^A-Za-z0-9])((?:sk-proj-|sk-)[A-Za-z0-9_-]{32,})(?:$|[^A-Za-z0-9])",
            )
            .unwrap(),
        },
        SecretPattern {
            name: "Gemini API Key",
            regex: Regex::new(r"(?:^|[^A-Za-z0-9])(AIza[0-9A-Za-z_-]{35})(?:$|[^A-Za-z0-9])")
                .unwrap(),
        },
    ]
}
