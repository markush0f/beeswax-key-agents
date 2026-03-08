//! Background thread integration for the `vault-core` scanning suite.
//!
//! This module handles launching discrete std threads and piping the `vault-core` streams
//! back to the main UI loop via MPSC channels.

use std::sync::mpsc;

use vault_core::{
    KeyMatch, scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming,
    scan_project_files_for_keys_streaming,
};

/// A collection of receiving channels and thread handles constructed by the scanner.
pub struct ScanChannels {
    /// Channel for receiving matches from .env configurations.
    pub env_rx: mpsc::Receiver<KeyMatch>,
    /// Channel for receiving matches from hidden IDE folders (.vscode, .idea).
    pub ide_rx: mpsc::Receiver<KeyMatch>,
    /// Channel for receiving heuristic hardcoded matches from all other source files.
    pub files_rx: mpsc::Receiver<KeyMatch>,
    /// Thread handle for the `.env` scan.
    pub env_handle: std::thread::JoinHandle<()>,
    /// Thread handle for the IDE files scan.
    pub ide_handle: std::thread::JoinHandle<()>,
    /// Thread handle for the generic project files scan.
    pub files_handle: std::thread::JoinHandle<()>,
}

/// Spawns 3 dedicated threads to recursively scan the given path concurrently.
///
/// Returns a `ScanChannels` struct which the master UI loop can poll for live-updates.
pub fn spawn_scanners(scan_path: String) -> ScanChannels {
    let (env_tx, env_rx) = mpsc::channel::<KeyMatch>();
    let (ide_tx, ide_rx) = mpsc::channel::<KeyMatch>();
    let (files_tx, files_rx) = mpsc::channel::<KeyMatch>();

    let scan_path_for_env = scan_path.clone();
    let env_handle = std::thread::spawn(move || {
        scan_env_for_keys_streaming(&scan_path_for_env, move |m| {
            let _ = env_tx.send(m);
        });
    });

    let scan_path_for_ide = scan_path.clone();
    let ide_handle = std::thread::spawn(move || {
        scan_ide_files_for_keys_streaming(&scan_path_for_ide, move |m| {
            let _ = ide_tx.send(m);
        });
    });

    let scan_path_for_files = scan_path;
    let files_handle = std::thread::spawn(move || {
        scan_project_files_for_keys_streaming(&scan_path_for_files, move |m| {
            if m.hardcoded {
                let _ = files_tx.send(m);
            }
        });
    });

    ScanChannels {
        env_rx,
        ide_rx,
        files_rx,
        env_handle,
        ide_handle,
        files_handle,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_scanners_channels() {
        let channels = spawn_scanners(".".to_string());

        // Ensure channels are created and handles exist
        // (We don't need to join them here as they will drop naturally or we can just let them run)
        assert!(channels.env_rx.try_recv().is_err());
        assert!(channels.ide_rx.try_recv().is_err());
        assert!(channels.files_rx.try_recv().is_err());
    }
}
