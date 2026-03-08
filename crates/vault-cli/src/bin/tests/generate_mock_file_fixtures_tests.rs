use super::{file_extension, fixture_relative_path, is_valid_prefix};

#[test]
fn rotates_extensions_for_regular_files() {
    assert_eq!(file_extension(1), "ts");
    assert_eq!(file_extension(2), "py");
    assert_eq!(file_extension(3), "json");
    assert_eq!(file_extension(4), "yaml");
    assert_eq!(file_extension(5), "toml");
    assert_eq!(file_extension(6), "rs");
}

#[test]
fn embeds_prefix_in_file_name() {
    let path = fixture_relative_path("vault_file", 1);
    assert!(path.to_string_lossy().contains("vault_file_001"));
}

#[test]
fn accepts_only_safe_prefix_characters() {
    assert!(is_valid_prefix("vault_file"));
    assert!(is_valid_prefix("vault-file-2"));
    assert!(!is_valid_prefix("vault file"));
    assert!(!is_valid_prefix("../vault_file"));
}
