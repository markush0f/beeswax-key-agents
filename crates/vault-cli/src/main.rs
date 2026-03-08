//! `vault-cli` — Interactive terminal dashboard for the Vault secret scanner.
//!
//! This binary coordinates three components:
//!
//! 1. **CLI parsing**: Resolves the scan path from the `--path` flag or an interactive prompt.
//! 2. **Scanner threads**: Delegates to [`scanner::spawn_scanners`] to run three concurrent
//!    `vault-core` scans (env files, IDE configs, and source code) in the background.
//! 3. **TUI event loop**: Hands control to [`app::App::run`], which drives the `ratatui`
//!    dashboard until the user exits.
//!
//! After the TUI exits, the scanner threads are joined and a one-line summary is printed
//! to stdout showing the total number of findings per tab.

mod app;
mod scanner;
mod state;
mod ui;

#[cfg(test)]
mod tests;

use clap::Parser;
use colored::*;
use directories::UserDirs;
use inquire::Text;
use std::process;

use crate::app::App;
use crate::scanner::spawn_scanners;
use crate::state::AppState;

/// Command-line arguments accepted by the `bkad` binary.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional directory path to scan. If omitted, an interactive prompt is shown.
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("{}", "======================================".cyan().bold());
    println!(
        "{}",
        "      Vault CLI - Secret Scanner      ".green().bold()
    );
    println!("{}", "======================================".cyan().bold());
    println!();

    let scan_path = resolve_scan_path(args.path);
    let scan = spawn_scanners(scan_path.clone());

    let state = AppState::new(scan_path);
    let state = App::run(state, scan.env_rx, scan.ide_rx, scan.files_rx);

    let _ = scan.env_handle.join();
    let _ = scan.ide_handle.join();
    let _ = scan.files_handle.join();

    println!(
        "{} .env: {}, IDES: {}, FILES: {}",
        "Summary:".cyan().bold(),
        state.env.len(),
        state.ides.len(),
        state.files.len()
    );
}

/// Resolves the directory to scan from the CLI flag or an interactive prompt.
///
/// Priority:
/// 1. If `--path` was provided, use it directly.
/// 2. Otherwise, show an `inquire` text prompt with the user's home directory
///    as the default, falling back to `"."` if the home directory cannot be resolved.
///
/// If the user cancels the interactive prompt (e.g., via Ctrl+C), the process
/// exits with code 1 and a user-friendly error message.
///
/// # Arguments
///
/// * `flag_path` - Value of the `--path` CLI argument, if provided.
fn resolve_scan_path(flag_path: Option<String>) -> String {
    match flag_path {
        Some(p) => p,
        None => {
            let default_path = if let Some(user_dirs) = UserDirs::new() {
                user_dirs.home_dir().to_string_lossy().to_string()
            } else {
                ".".to_string()
            };

            let prompt_result = Text::new("Directory to scan:")
                .with_default(&default_path)
                .prompt();

            match prompt_result {
                Ok(path) => path,
                Err(_) => {
                    eprintln!("{} Operation canceled by user.", "✖".red());
                    process::exit(1);
                }
            }
        }
    }
}
