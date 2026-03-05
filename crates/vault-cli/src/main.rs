use clap::Parser;
use colored::*;
use vault_core::scan_env_for_keys;

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
        "   Vault CLI - Escáner de Secretos    ".green().bold()
    );
    println!("{}", "======================================".cyan().bold());
    println!(
        "\nBuscando archivos .env y extrayendo secretos en: {} ...\n",
        args.path.yellow()
    );

    let results = scan_env_for_keys(&args.path);

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
