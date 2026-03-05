use clap::Parser;
use colored::*;
use vault_core::find_env_files;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directorio a escanear (por defecto: el directorio actual)
    #[arg(short, long, default_value = ".")]
    path: String,
}

fn main() {
    let args = Args::parse();

    println!("{}", "======================================".cyan().bold());
    println!(
        "{}",
        "   Vault CLI - Escáner de .env        ".green().bold()
    );
    println!("{}", "======================================".cyan().bold());
    println!("\nBuscando archivos .env en: {} ...\n", args.path.yellow());

    let env_files = find_env_files(&args.path);

    if env_files.is_empty() {
        println!(
            "{}",
            "✔ No se encontraron archivos .env en el directorio especificado.".green()
        );
    } else {
        println!(
            "{} Se encontraron {} archivo(s) .env potencialmente expuestos:\n",
            "⚠".red().bold(),
            env_files.len().to_string().red()
        );
        for file in env_files {
            println!("  ➜ {}", file.display().to_string().red());
        }
        println!();
    }
}
