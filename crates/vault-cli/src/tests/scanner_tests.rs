use crate::scanner::*;

#[test]
fn test_spawn_scanners_channels() {
    let channels = spawn_scanners(".".to_string());

    // Ensure channels are created and handles exist
    assert!(channels.env_rx.try_recv().is_err());
    assert!(channels.ide_rx.try_recv().is_err());
    assert!(channels.files_rx.try_recv().is_err());
}
