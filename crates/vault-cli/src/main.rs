use clap::Parser;
use colored::*;
use directories::UserDirs;
use inquire::Text;
use std::process;
use vault_core::scan_env_for_keys;

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

    // Si pasaron el flag `--path`, lo usamos directo; si no, preguntamos.
    let scan_path = match args.path {
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
    };

    let results = scan_env_for_keys(&scan_path);

    if results.is_empty() {
        println!(
            "{}",
            "✔ No se encontraron API keys expuestas en ningún .env.".green()
        );
    } else {
        println!(
            "{} ¡Alerta! Se encontraron {} API keys expuestas:\n",
            "⚠".red().bold(),
            results.len().to_string().red()
        );
        for m in results {
            println!(
                "  [{}] {}",
                "Proveedor".yellow().bold(),
                m.provider.magenta().bold()
            );
            println!(
                "  [{}] {}",
                "Archivo".blue().bold(),
                m.file_path.display().to_string().cyan()
            );
            println!("  [{}] {}", "Línea".blue().bold(), m.line_number);
            println!("  [{}] {}\n", "Clave".blue().bold(), m.key.red());
        }
    }
}
