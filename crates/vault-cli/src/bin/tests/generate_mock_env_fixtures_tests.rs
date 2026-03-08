use super::{env_file_name, is_valid_prefix};

#[test]
fn keeps_env_prefix_in_generated_file_names() {
    assert_eq!(env_file_name("vault_env", 1), ".env.vault_env_001.local");
    assert_eq!(env_file_name("vault_env", 5), ".env.vault_env_005");
}

#[test]
fn accepts_only_safe_prefix_characters() {
    assert!(is_valid_prefix("vault_env"));
    assert!(is_valid_prefix("vault-env-2"));
    assert!(!is_valid_prefix("vault env"));
    assert!(!is_valid_prefix("../vault_env"));
}
