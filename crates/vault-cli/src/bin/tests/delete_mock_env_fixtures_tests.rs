use std::path::Path;

use super::{is_generated_env_file, is_valid_prefix};

#[test]
fn matches_generated_env_file_names() {
    assert!(is_generated_env_file(
        Path::new(".env.vault_env_001.local"),
        "vault_env"
    ));
    assert!(is_generated_env_file(
        Path::new(".env.vault_env_002.production"),
        "vault_env"
    ));
    assert!(!is_generated_env_file(Path::new(".env.local"), "vault_env"));
    assert!(!is_generated_env_file(
        Path::new(".env.other_prefix_001"),
        "vault_env"
    ));
}

#[test]
fn validates_prefix_safely() {
    assert!(is_valid_prefix("vault_env"));
    assert!(is_valid_prefix("vault-env-2"));
    assert!(!is_valid_prefix("vault env"));
    assert!(!is_valid_prefix("../vault_env"));
}
