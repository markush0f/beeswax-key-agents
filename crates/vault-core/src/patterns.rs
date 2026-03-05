use regex::Regex;

pub struct SecretPattern {
    pub name: &'static str,
    pub regex: Regex,
}

pub fn get_patterns() -> Vec<SecretPattern> {
    vec![
        SecretPattern {
            name: "OpenAI API Key",
            regex: Regex::new(r"(sk-proj-[a-zA-Z0-9_\-]{20,}|sk-[a-zA-Z0-9_\-]{20,})").unwrap(),
        },
        SecretPattern {
            name: "Gemini API Key",
            // Las API Keys de Gemini (Google AI Studio) típicamente comienzan con AIza y tienen 39 caracteres en total.
            regex: Regex::new(r"(AIza[0-9A-Za-z_\-]{35})").unwrap(),
        },
    ]
}
