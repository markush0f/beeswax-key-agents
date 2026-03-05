use clap::Parser;
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Attribute, SetAttribute},
    terminal::{self, ClearType},
};
use directories::UserDirs;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use std::process;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use vault_core::{scan_env_for_keys_streaming, scan_ide_files_for_keys_streaming};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewTab {
    Env,
    Ides,
    Exit,
}

fn choose_tab_with_left_right(
    env_count: usize,
    ide_done: &AtomicBool,
    ide_results: &Mutex<Vec<vault_core::KeyMatch>>,
) -> ViewTab {
    let mut selected = ViewTab::Env;
    let mut stdout = std::io::stdout();

    let _ = terminal::enable_raw_mode();
    let _ = execute!(stdout, cursor::Hide);

    loop {
        let ide_text = if ide_done.load(Ordering::Relaxed) {
            let n = ide_results.lock().map(|g| g.len()).unwrap_or(0);
            format!("IDES ({n})")
        } else {
            "IDES (cargando...)".to_string()
        };
        let env_text = format!("ENV (.env) ({env_count})");

        let (cols, _) = terminal::size().unwrap_or((80, 0));
        let cols = cols as usize;
        let spacer_len = cols.saturating_sub(env_text.len() + ide_text.len()).max(1);
        let spacer = " ".repeat(spacer_len);

        let _ = execute!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine)
        );

        if selected == ViewTab::Env {
            let _ = execute!(stdout, SetAttribute(Attribute::Reverse));
        }
        let _ = std::io::Write::write_all(&mut stdout, env_text.as_bytes());
        if selected == ViewTab::Env {
            let _ = execute!(stdout, SetAttribute(Attribute::Reset));
        }

        let _ = std::io::Write::write_all(&mut stdout, spacer.as_bytes());

        if selected == ViewTab::Ides {
            let _ = execute!(stdout, SetAttribute(Attribute::Reverse));
        }
        let _ = std::io::Write::write_all(&mut stdout, ide_text.as_bytes());
        if selected == ViewTab::Ides {
            let _ = execute!(stdout, SetAttribute(Attribute::Reset));
        }

        let _ = std::io::Write::flush(&mut stdout);

        if event::poll(std::time::Duration::from_millis(120)).unwrap_or(false) {
            if let Ok(Event::Key(k)) = event::read() {
                if k.kind != KeyEventKind::Press {
                    continue;
                }

                match k.code {
                    KeyCode::Left => selected = ViewTab::Env,
                    KeyCode::Right => selected = ViewTab::Ides,
                    KeyCode::Enter => break selected,
                    KeyCode::Esc => break ViewTab::Exit,
                    KeyCode::Char('q') => break ViewTab::Exit,
                    _ => {}
                }
            }
        }
    }
}

fn leave_raw_mode() {
    let mut stdout = std::io::stdout();
    let _ = execute!(
        stdout,
        SetAttribute(Attribute::Reset),
        cursor::Show,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine)
    );
    let _ = terminal::disable_raw_mode();
}

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
    spinner.set_message(format!("Escaneando .env en {}", scan_path));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let ide_results: Arc<Mutex<Vec<vault_core::KeyMatch>>> = Arc::new(Mutex::new(Vec::new()));
    let ide_done = Arc::new(AtomicBool::new(false));

    let scan_path_for_ide = scan_path.clone();
    let ide_results_for_thread = ide_results.clone();
    let ide_done_for_thread = ide_done.clone();
    let ide_handle = thread::spawn(move || {
        scan_ide_files_for_keys_streaming(&scan_path_for_ide, |m| {
            if let Ok(mut guard) = ide_results_for_thread.lock() {
                guard.push(m);
            }
        });
        ide_done_for_thread.store(true, Ordering::Relaxed);
    });

    let spinner_for_cb = spinner.clone();
    let mut env_results = Vec::new();
    scan_env_for_keys_streaming(&scan_path, |m| {
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

        env_results.push(m);
    });

    spinner.finish_and_clear();

    let mut ide_handle = Some(ide_handle);

    if env_results.is_empty() {
        println!(
            "\n{}",
            "✔ No se encontraron API keys expuestas en ningún .env.".green()
        );
    } else {
        println!(
            "\n{} Se encontraron {} coincidencias en .env.\n",
            "⚠".red().bold(),
            env_results.len().to_string().red()
        );
    }

    println!("←/→ cambia pestaña, Enter abre, Esc sale.");

    loop {
        let tab = choose_tab_with_left_right(env_results.len(), &ide_done, &ide_results);
        leave_raw_mode();
        println!();

        match tab {
            ViewTab::Exit => break,
            ViewTab::Env => {
                if env_results.is_empty() {
                    println!("{}", "No hay resultados en .env.".cyan());
                    continue;
                }

                let mut options: Vec<String> = env_results
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
                options.push("[Volver]".to_string());

                let _ = Select::new(".env:", options).with_page_size(15).prompt();
            }
            ViewTab::Ides => {
                if !ide_done.load(Ordering::Relaxed) {
                    let wait_spinner = ProgressBar::new_spinner();
                    wait_spinner.set_style(
                        ProgressStyle::with_template("{spinner} {msg}")
                            .unwrap()
                            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
                    );
                    wait_spinner.set_message("Terminando escaneo de IDES...".to_string());
                    wait_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

                    if let Some(h) = ide_handle.take() {
                        let _ = h.join();
                    }
                    wait_spinner.finish_and_clear();
                }

                let ide_list = ide_results.lock().map(|g| g.clone()).unwrap_or_default();
                if ide_list.is_empty() {
                    println!("{}", "No se encontraron claves en IDES.".cyan());
                    continue;
                }

                let mut ide_options: Vec<String> = ide_list
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

                ide_options.push("[Volver]".to_string());

                let _ = Select::new("IDES:", ide_options)
                    .with_page_size(15)
                    .prompt();
            }
        }
    }
}
