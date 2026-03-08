use crate::scan::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_scan_env_files() {
    let dir = tempdir().unwrap();
    let env_path = dir.path().join(".env");
    fs::write(
        &env_path,
        "OPENAI_KEY=sk-proj-12345678901234567890123456789012",
    )
    .unwrap();

    let matches = scan_env_for_keys(dir.path().to_str().unwrap());
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].provider, "OpenAI API Key");
}

#[test]
fn test_scan_all_files_ignores_node_modules() {
    let dir = tempdir().unwrap();
    let nm_dir = dir.path().join("node_modules");
    fs::create_dir(&nm_dir).unwrap();
    fs::write(
        nm_dir.join("secret.txt"),
        "sk-proj-12345678901234567890123456789012",
    )
    .unwrap();

    let matches = scan_all_files_for_keys(dir.path().to_str().unwrap());
    assert!(matches.is_empty()); // Should be ignored
}
