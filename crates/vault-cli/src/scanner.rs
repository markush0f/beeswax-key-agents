use std::sync::mpsc;

use vault_core::{
    KeyMatch, scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming,
    scan_project_files_for_keys_streaming,
};

pub struct ScanChannels {
    pub env_rx: mpsc::Receiver<KeyMatch>,
    pub ide_rx: mpsc::Receiver<KeyMatch>,
    pub files_rx: mpsc::Receiver<KeyMatch>,
    pub env_handle: std::thread::JoinHandle<()>,
    pub ide_handle: std::thread::JoinHandle<()>,
    pub files_handle: std::thread::JoinHandle<()>,
}

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
