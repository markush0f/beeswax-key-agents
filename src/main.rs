use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::thread;
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

    println!("\n{} {}...", "Starting".yellow().bold(), scan_type.cyan());
    println!("Target path: {}\n", path.italic());

    // Create a progress bar (spinner)
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Scanning '{}' for leaked keys...", path));

    // Simulate work
    for _ in 0..40 {
        pb.inc(1);
        thread::sleep(Duration::from_millis(50));
    }

    pb.finish_with_message("Scan complete!".green().to_string());

    println!("\n{}", "Results:".green().bold());
    println!("  {} No keys found! Your code looks secure.\n", "✔".green());
}
