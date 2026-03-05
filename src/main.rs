use agent_key_detector::models::scan::ScanType;
use agent_key_detector::services::scanner::ScannerService;
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::time::Duration;

/// Agent Key Detector CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory or file to scan for keys
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("{}", "======================================".cyan().bold());
    println!(
        "{}",
        "   Agent Key Detector - Welcome!      ".green().bold()
    );
    println!("{}", "======================================".cyan().bold());
    println!();

    // Interactively ask for the path if it wasn't provided
    let path = match args.path {
        Some(p) => p,
        None => Text::new("Enter the directory or file to scan:")
            .with_default(".")
            .prompt()
            .unwrap_or_else(|_| ".".to_string()),
    };

    // Ask for the type of scan
    let scan_types = vec!["Quick Scan", "Deep Scan", "Custom Scan"];
    let scan_type = Select::new("Select the type of scan:", scan_types)
        .prompt()
        .unwrap_or_else(|_| "Quick Scan");

    let selected_scan_type = match scan_type {
        "Deep Scan" => ScanType::Deep,
        "Custom Scan" => ScanType::Custom,
        _ => ScanType::Quick,
    };

    println!("\n{} {}...", "Starting".yellow().bold(), scan_type.cyan());
    println!("Target path: {}\n", path.italic());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!("Scanning '{}' for leaked keys...", path));

    // Initialize and run the scanner service
    let scanner = ScannerService::new();
    let result = scanner.scan_path(&path, selected_scan_type);

    pb.finish_with_message("Scan complete!".green().to_string());

    match result {
        Ok(res) => {
            println!("\n{}", "Results:".green().bold());
            println!("  Files scanned: {}", res.files_scanned.to_string().cyan());
            if res.matches.is_empty() {
                println!("  {} No keys found! Your code looks secure.\n", "✔".green());
            } else {
                println!(
                    "  {} Found {} potential keys/secrets:\n",
                    "⚠".red().bold(),
                    res.matches.len().to_string().red()
                );
                for m in res.matches {
                    println!(
                        "    [{}] {}:{}",
                        m.key_type.yellow(),
                        m.file_path,
                        m.line_number
                    );
                    println!("    ➜ {}\n", m.matched_content.red());
                }
            }
        }
        Err(e) => {
            println!("\n  {} Error scanning: {}\n", "✖".red(), e);
        }
    }
}
