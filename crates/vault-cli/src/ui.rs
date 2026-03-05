use std::io::Write;

use crossterm::{
    cursor, execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::state::{AppState, Tab};

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> TerminalGuard {
        let mut stdout = std::io::stdout();
        let _ = terminal::enable_raw_mode();
        let _ = execute!(stdout, EnterAlternateScreen, cursor::Hide);
        TerminalGuard
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = std::io::stdout();
        let _ = execute!(
            stdout,
            SetAttribute(Attribute::Reset),
            cursor::Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

pub struct Renderer;

impl Renderer {
    pub fn render(stdout: &mut std::io::Stdout, state: &AppState, tick: u64) {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        let w = w as usize;
        let h = h as usize;

        let _ = queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        );
        Self::render_header(stdout, state, w, tick);
        Self::render_body(stdout, state, w, h);
        Self::render_status(stdout, state, w, h);
        let _ = stdout.flush();
    }

    pub fn body_height() -> usize {
        let (_, height) = terminal::size().unwrap_or((80, 24));
        height.saturating_sub(3) as usize
    }

    fn render_header(stdout: &mut std::io::Stdout, state: &AppState, width: usize, tick: u64) {
        let env_state = if state.env.done {
            "OK"
        } else {
            spinner_ascii(tick)
        };
        let ide_state = if state.ides.done {
            "OK"
        } else {
            spinner_ascii(tick)
        };

        let left = format!(" ENV (.env) {} [{}] ", state.env.len(), env_state);
        let right = format!(" IDES {} [{}] ", state.ides.len(), ide_state);

        let right_w = right.chars().count().min(width);
        let right_start = width.saturating_sub(right_w);
        let left_w = right_start;

        let _ = queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::CurrentLine)
        );

        if state.tab == Tab::Env {
            let _ = queue!(stdout, SetAttribute(Attribute::Reverse));
        }
        let _ = queue!(stdout, Print(trunc_to_width(&left, left_w)));
        let _ = queue!(stdout, SetAttribute(Attribute::Reset));

        let _ = queue!(stdout, cursor::MoveTo(right_start as u16, 0));
        if state.tab == Tab::Ides {
            let _ = queue!(stdout, SetAttribute(Attribute::Reverse));
        }
        let _ = queue!(stdout, Print(trunc_to_width(&right, right_w)));
        let _ = queue!(stdout, SetAttribute(Attribute::Reset));

        let _ = queue!(stdout, cursor::MoveToNextLine(1));

        let hint = format!(
            "Ruta: {} | ←/→/Tab cambia, e/i directo, ↑/↓ mueve, PgUp/PgDn, Home/End, q/Esc sale",
            state.scan_path
        );
        let _ = queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            Print(trunc_to_width(&hint, width))
        );
        let _ = queue!(stdout, cursor::MoveToNextLine(1));
    }

    fn render_body(stdout: &mut std::io::Stdout, state: &AppState, width: usize, height: usize) {
        if height < 3 {
            return;
        }
        let body_h = height - 3;

        let (items, scroll, selected, empty_msg) = match state.tab {
            Tab::Env => (
                &state.env.items,
                state.env.scroll(),
                state.env.selected(),
                "Sin resultados en .env (todavia).",
            ),
            Tab::Ides => (
                &state.ides.items,
                state.ides.scroll(),
                state.ides.selected(),
                "Sin resultados en IDES (todavia).",
            ),
        };

        if items.is_empty() {
            for _ in 0..body_h {
                let _ = queue!(stdout, Print(pad_trunc(empty_msg, width)));
                let _ = queue!(stdout, cursor::MoveToNextLine(1));
            }
            return;
        }

        for row in 0..body_h {
            let idx = scroll + row;
            if idx >= items.len() {
                let _ = queue!(stdout, Print(" ".repeat(width)));
                let _ = queue!(stdout, cursor::MoveToNextLine(1));
                continue;
            }

            let line = format_match_line(&items[idx]);
            if idx == selected {
                let _ = queue!(stdout, SetAttribute(Attribute::Reverse));
            }
            let _ = queue!(stdout, Print(pad_trunc(&line, width)));
            let _ = queue!(stdout, SetAttribute(Attribute::Reset));
            let _ = queue!(stdout, cursor::MoveToNextLine(1));
        }
    }

    fn render_status(stdout: &mut std::io::Stdout, state: &AppState, width: usize, height: usize) {
        if height == 0 {
            return;
        }
        let y = (height - 1) as u16;
        let _ = queue!(
            stdout,
            cursor::MoveTo(0, y),
            terminal::Clear(ClearType::CurrentLine)
        );

        let active = state.active_list();
        let status = if active.is_empty() {
            if active.done {
                "Listo. (0 resultados)".to_string()
            } else {
                "Escaneando...".to_string()
            }
        } else {
            format!(
                "Sel {}/{} | Scroll {} | {}",
                active.selected() + 1,
                active.len(),
                active.scroll() + 1,
                if active.done {
                    "Listo"
                } else {
                    "Escaneando..."
                }
            )
        };

        let _ = queue!(stdout, Print(pad_trunc(&status, width)));
    }
}

fn spinner_ascii(tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[(tick as usize) % FRAMES.len()]
}

fn format_match_line(m: &vault_core::KeyMatch) -> String {
    let hardcoded_label = if m.hardcoded {
        "HARDCODEADA"
    } else {
        "posible referencia"
    };
    format!(
        "[{}] {} : L{} -> {} [{}]",
        m.provider,
        m.file_path.display(),
        m.line_number,
        m.key,
        hardcoded_label
    )
}

fn pad_trunc(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut count = 0usize;
    for ch in s.chars() {
        if count >= width {
            break;
        }
        out.push(ch);
        count += 1;
    }
    if count < width {
        out.push_str(&" ".repeat(width - count));
    }
    out
}

fn trunc_to_width(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut count = 0usize;
    for ch in s.chars() {
        if count >= width {
            break;
        }
        out.push(ch);
        count += 1;
    }
    out
}
