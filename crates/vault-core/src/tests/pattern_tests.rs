use crate::patterns::get_patterns;

#[test]
fn test_patterns_metadata() {
    let patterns = get_patterns();
    assert!(!patterns.is_empty());

    for p in patterns {
        assert!(!p.name.is_empty());
        assert!(!p.short_name.is_empty());
    }
}

#[test]
fn test_openai_pattern() {
    let patterns = get_patterns();
    let openai = patterns
        .iter()
        .find(|p| p.name == "OpenAI API Key")
        .unwrap();

    // Valid sk-proj
    assert!(
        openai
            .regex
            .is_match(" sk-proj-12345678901234567890123456789012 ")
    );
    // Valid sk-
    assert!(
        openai
            .regex
            .is_match(" sk-12345678901234567890123456789012 ")
    );

    // Exclusion check (OpenRouter vs OpenAI)
    let or_key = "sk-or-v1-1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    assert!(openai.regex.is_match(or_key)); // Regex matches the format
    assert!(!openai.allows_key(or_key)); // allowed_key filters it out
}

#[test]
fn test_gemini_pattern() {
    let patterns = get_patterns();
    let gemini = patterns
        .iter()
        .find(|p| p.name == "Gemini API Key")
        .unwrap();

    assert!(
        gemini
            .regex
            .is_match(" AIzaSyA12345678901234567890123456789012 ")
    );
}
