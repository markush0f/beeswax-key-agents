use clap::Parser;
use colored::*;
use directories::UserDirs;
use inquire::{Select, Text};
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
            "\n{}",
            "✔ No se encontraron API keys expuestas en ningún .env.".green()
        );
    } else {
        println!(
            "\n{} ¡Alerta! Se encontraron {} API keys expuestas.\n",
            "⚠".red().bold(),
            results.len().to_string().red()
        );

        // Mapear los resultados a una lista de strings para inquire::Select
        let mut options: Vec<String> = results
            .iter()
            .map(|m| {
                format!(
                    "[{}] {} : L{} ➜ {}",
                    m.provider,
                    m.file_path.display(),
                    m.line_number,
                    m.key
                )
            })
            .collect();

        options.push("👉 [Salir]".to_string());

        let _ = Select::new(
            "Navega por los secretos encontrados (Usa ↑/↓, Enter para seleccionar):",
            options,
        )
        .with_page_size(15)
        .prompt();
    }
}
