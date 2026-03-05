use clap::Parser;
use colored::*;
use directories::UserDirs;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::process;
use vault_core::scan_all_files_for_keys_streaming;

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

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(format!("Escaneando archivos en {}", scan_path));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let spinner_for_cb = spinner.clone();
    let mut results = Vec::new();
    scan_all_files_for_keys_streaming(&scan_path, |m| {
        let hardcoded_label = if m.hardcoded {
            "HARDCODEADA"
        } else {
            "posible referencia"
        };

        spinner_for_cb.println(format!(
            "[{}] {} : L{} ➜ {} [{}]",
            m.provider,
            m.file_path.display(),
            m.line_number,
            m.key,
            hardcoded_label
        ));

        results.push(m);
    });

    if results.is_empty() {
        spinner.finish_and_clear();
        println!(
            "\n{}",
            "✔ No se encontraron API keys expuestas en los archivos escaneados.".green()
        );
    } else {
        spinner.finish_and_clear();
        println!(
            "\n{} ¡Alerta! Se encontraron {} API keys expuestas en archivos del proyecto.\n",
            "⚠".red().bold(),
            results.len().to_string().red()
        );

        // Mapear los resultados a una lista de strings para inquire::Select
        let mut options: Vec<String> = results
            .iter()
            .map(|m| {
                let hardcoded_label = if m.hardcoded {
                    "HARDCODEADA"
                } else {
                    "posible referencia"
                };

                format!(
                    "[{}] {} : L{} ➜ {} [{}]",
                    m.provider,
                    m.file_path.display(),
                    m.line_number,
                    m.key,
                    hardcoded_label
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
