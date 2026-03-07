mod app;
mod scanner;
mod state;
mod ui;

use clap::Parser;
use colored::*;
use directories::UserDirs;
use inquire::Text;
use std::process;

use crate::app::App;
use crate::scanner::spawn_scanners;
use crate::state::AppState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directorio a escanear (por defecto: interactivo, sugiere el directorio home del usuario)
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("{}", "======================================".cyan().bold());
    println!(
        "{}",
        "   Vault CLI - Escáner de Secretos    ".green().bold()
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
        "Resumen:".cyan().bold(),
        state.env.len(),
        state.ides.len(),
        state.files.len()
    );
}

fn resolve_scan_path(flag_path: Option<String>) -> String {
    match flag_path {
        Some(p) => p,
        None => {
            let default_path = if let Some(user_dirs) = UserDirs::new() {
                user_dirs.home_dir().to_string_lossy().to_string()
            } else {
                ".".to_string()
            };

            let prompt_result = Text::new("Directorio a escanear:")
                .with_default(&default_path)
                .prompt();

            match prompt_result {
                Ok(path) => path,
                Err(_) => {
                    eprintln!("{} Operación cancelada por el usuario.", "✖".red());
                    process::exit(1);
                }
            }
        }
    }
}
