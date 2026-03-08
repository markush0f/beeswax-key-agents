//! Background scanner thread launcher and channel management.
//!
//! This module bridges `vault-core`'s blocking streaming API with the non-blocking
//! TUI event loop by running each scanner on a dedicated OS thread and connecting
//! it to the main thread via standard MPSC channels.
//!
//! ## Thread Model
//!
//! ```text
//!  ┌────────────────────────┐    env_tx ──► env_rx
//!  │   scan_env_for_keys_   │
//!  │   streaming (thread 1) │
//!  └────────────────────────┘
//!
//!  ┌────────────────────────┐    ide_tx ──► ide_rx
//!  │   scan_ide_files_for_  │
//!  │   keys_streaming (t2)  │
//!  └────────────────────────┘
//!
//!  ┌────────────────────────┐  files_tx ──► files_rx
//!  │   scan_project_files_  │
//!  │   for_keys_streaming   │
//!  │   (thread 3, filtered) │
//!  └────────────────────────┘
//! ```
//!
//! The main event loop polls all three receivers with [`std::sync::mpsc::Receiver::try_recv`],
//! pushing every received [`KeyMatch`] into the corresponding [`AppState`] list.
//!
//! ## Filter on the Files Channel
//!
//! The project-files scanner emits all matches, but the thread wrapper in this module
//! forwards only matches where `hardcoded == true` to the `files_tx` channel. This
//! avoids false positives from dynamically referenced keys in large codebases.

use std::sync::mpsc;

use vault_core::{
    KeyMatch, scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming,
    scan_project_files_for_keys_streaming,
};

/// Bundles all MPSC channel endpoints and thread handles created by [`spawn_scanners`].
///
/// The caller owns this struct for the duration of the application. Dropping the
/// receivers disconnects the channels; the scanner threads will finish naturally
/// when they reach the end of their directory walk.
pub struct ScanChannels {
    /// Receiving end for matches found in `.env*` files.
    pub env_rx: mpsc::Receiver<KeyMatch>,
    /// Receiving end for matches found in IDE configuration directories.
    pub ide_rx: mpsc::Receiver<KeyMatch>,
    /// Receiving end for hardcoded matches found in project source files.
    pub files_rx: mpsc::Receiver<KeyMatch>,
    /// Join handle for the env scanner thread.
    pub env_handle: std::thread::JoinHandle<()>,
    /// Join handle for the IDE scanner thread.
    pub ide_handle: std::thread::JoinHandle<()>,
    /// Join handle for the project files scanner thread.
    pub files_handle: std::thread::JoinHandle<()>,
}

/// Launches three concurrent scanner threads and returns their communication channels.
///
/// Each thread runs one of the `vault-core` streaming scan functions against `scan_path`
/// and forwards discovered [`KeyMatch`]es through its MPSC sender. The senders are
/// moved into the threads and dropped when the scan completes, which closes the channel
/// and allows the event loop to detect that the thread has finished.
///
/// ## Hardcoded Filter
///
/// The project-files thread only sends a match if `m.hardcoded == true`. This reduces
/// noise from keys that are referenced by variable rather than being literal strings.
///
/// # Arguments
///
/// * `scan_path` - The root directory to scan. Cloned into each thread.
///
/// # Returns
///
/// A [`ScanChannels`] bundle ready to be handed to [`App::run`](crate::app::App::run).
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
