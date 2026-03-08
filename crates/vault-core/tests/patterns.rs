use vault_core::patterns::get_patterns;

#[test]
fn matches_openrouter_keys() {
    let patterns = get_patterns();
    let openrouter = patterns
        .iter()
        .find(|p| p.name == "OpenRouter API Key")
        .expect("openrouter pattern should exist");

    let line = r#"OPENROUTER_API_KEY="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""#;
    assert!(openrouter.first_capture(line).is_some());
}

#[test]
fn does_not_classify_openrouter_as_openai() {
    let patterns = get_patterns();
    let openai = patterns
        .iter()
        .find(|p| p.name == "OpenAI API Key")
        .expect("openai pattern should exist");

    let line = r#"OPENROUTER_API_KEY="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""#;
    assert!(openai.first_capture(line).is_none());
}

#[test]
fn matches_anthropic_keys() {
    let patterns = get_patterns();
    let anthropic = patterns
        .iter()
        .find(|p| p.name == "Anthropic API Key")
        .expect("anthropic pattern should exist");

    let line = r#"ANTHROPIC_API_KEY="sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWX1234567890abcdEFGH""#;
    assert!(anthropic.first_capture(line).is_some());
}

#[test]
fn matches_grok_keys_from_xai_env_vars() {
    let patterns = get_patterns();
    let grok = patterns
        .iter()
        .find(|p| p.name == "Grok API Key")
        .expect("grok pattern should exist");

    let line = r#"XAI_API_KEY="w3p7p5quYqPlx7x_-B6N5Jb4M1i8wNzfF3bxJ6e5V9hGm1Qa""#;
    assert!(grok.first_capture(line).is_some());
}

#[test]
fn does_not_match_grok_placeholders() {
    let patterns = get_patterns();
    let grok = patterns
        .iter()
        .find(|p| p.name == "Grok API Key")
        .expect("grok pattern should exist");

    let line = r#"XAI_API_KEY="your_api_key""#;
    assert!(grok.first_capture(line).is_none());
}

#[test]
fn does_not_match_non_key_tokens() {
    let patterns = get_patterns();
    let anthropic = patterns
        .iter()
        .find(|p| p.name == "Anthropic API Key")
        .expect("anthropic pattern should exist");

    let line = "'asterisk-exception': {'id': 'Asterisk-exception'}";
    assert!(anthropic.first_capture(line).is_none());
}

#[test]
fn matches_ollama_keys() {
    let patterns = get_patterns();
    let ollama = patterns
        .iter()
        .find(|p| p.name == "Ollama API Key")
        .expect("ollama pattern should exist");

    let line = r#"OLLAMA_API_KEY="ollama_ABCDEFGHIJKLMNOPQRSTUVWXYZ123456""#;
    assert!(ollama.first_capture(line).is_some());
}

#[test]
fn matches_ollama_native_token_format() {
    let patterns = get_patterns();
    let ollama = patterns
        .iter()
        .find(|p| p.name == "Ollama API Key")
        .expect("ollama pattern should exist");

    let line = r#"OLLAMA_API_KEY="0972b6f6eb88495aa1f9f581189104f1._VH6UlaBHFRMsQ0vj-sRZYDq""#;
    assert!(ollama.first_capture(line).is_some());
}

#[test]
fn does_not_match_generic_ollama_text() {
    let patterns = get_patterns();
    let ollama = patterns
        .iter()
        .find(|p| p.name == "Ollama API Key")
        .expect("ollama pattern should exist");

    let line = "Use ollama serve to run local models";
    assert!(ollama.first_capture(line).is_none());
}

#[test]
fn matches_deepseek_keys() {
    let patterns = get_patterns();
    let deepseek = patterns
        .iter()
        .find(|p| p.name == "Deepseek API Key")
        .expect("deepseek pattern should exist");

    let line = r#"DEEPSEEK_API_KEY="sk-1234567890abcdef1234567890abcdef""#;
    assert!(deepseek.first_capture(line).is_some());
}

#[test]
fn does_not_match_shorter_deepseek_keys() {
    let patterns = get_patterns();
    let deepseek = patterns
        .iter()
        .find(|p| p.name == "Deepseek API Key")
        .expect("deepseek pattern should exist");

    let line = r#"DEEPSEEK_API_KEY="sk-1234567890abcdef1234567890abcde""#;
    assert!(deepseek.first_capture(line).is_none());
}
