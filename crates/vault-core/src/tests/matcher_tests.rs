use crate::matcher::*;
use crate::patterns::get_patterns;
use std::path::Path;

#[test]
fn test_mask_key() {
    // Test long key
    assert_eq!(
        crate::matcher::mask_key("sk-proj-1234567890abcdef"),
        "sk-proj-12...cdef"
    );

    // Test short key
    assert_eq!(crate::matcher::mask_key("short"), "****");
}

#[test]
fn test_is_hardcoded_in_line() {
    let key = "sk-1234567890";

    // Quoted
    assert!(crate::matcher::is_hardcoded_in_line(
        &format!("let x = \"{}\";", key),
        key
    ));
    assert!(crate::matcher::is_hardcoded_in_line(
        &format!("const y = '{}';", key),
        key
    ));
    assert!(crate::matcher::is_hardcoded_in_line(
        &format!("var z = `{}`;", key),
        key
    ));

    // Assignment
    assert!(crate::matcher::is_hardcoded_in_line(
        &format!("API_KEY={}", key),
        key
    ));
    assert!(crate::matcher::is_hardcoded_in_line(
        &format!("key: {}", key),
        key
    ));

    // Not hardcoded (e.g. part of a longer path or just random text)
    assert!(!crate::matcher::is_hardcoded_in_line(
        &format!("this is just some text with {}", key),
        key
    ));
}

#[test]
fn test_find_matches_streaming() {
    let patterns = get_patterns();
    let content = "Here is an OpenAI key: sk-proj-12345678901234567890123456789012\nAnd a Gemini one: AIzaSyA12345678901234567890123456789012";
    let path = Path::new("test.rs");

    let mut matches = Vec::new();
    find_matches_in_content_streaming_with_hash(path, content, &patterns, false, &mut |m, _| {
        matches.push(m)
    });

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].provider, "OpenAI API Key");
    assert_eq!(matches[1].provider, "Gemini API Key");
}
