use std::io;

use crossterm::{
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Frame, Terminal},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::state::{AppState, Tab};

pub const HEADER_HEIGHT: u16 = 5;
pub const FOOTER_HEIGHT: u16 = 1;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> io::Result<TerminalGuard> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn make_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn viewport_height(area: Rect) -> usize {
    // Body uses a `Block` with a TOP border, so inner list height is (body - 1).
    usize::from(area.height)
        .saturating_sub(usize::from(HEADER_HEIGHT + FOOTER_HEIGHT))
        .saturating_sub(1)
}

pub fn draw(frame: &mut Frame, state: &AppState, tick: u64) {
    let root = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(root);

    render_header(frame, state, chunks[0], tick);
    render_body(frame, state, chunks[1]);
    render_footer(frame, state, chunks[2]);
}

fn render_header(frame: &mut Frame, state: &AppState, area: Rect, tick: u64) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let title_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let subtitle_style = Style::default().fg(Color::Gray);

    let title =
        Paragraph::new(Line::from(Span::styled("VAULT", title_style))).alignment(Alignment::Center);
    frame.render_widget(title, rows[0]);

    let subtitle = Paragraph::new(Line::from(Span::styled("secret scanner", subtitle_style)))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, rows[1]);

    let spin = spinner_ascii(tick);
    let env_badge = if state.env.done {
        "DONE".to_string()
    } else {
        format!("SCAN {spin}")
    };
    let ide_badge = if state.ides.done {
        "DONE".to_string()
    } else {
        format!("SCAN {spin}")
    };

    let tabs = Tabs::new(vec![
        Line::from(format!(" ENV  {}  {env_badge} ", state.env.len())),
        Line::from(format!(" IDES {}  {ide_badge} ", state.ides.len())),
    ])
    .select(match state.tab {
        Tab::Env => 0,
        Tab::Ides => 1,
    })
    .highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .style(Style::default().fg(Color::DarkGray))
    .divider("  ");
    frame.render_widget(tabs, rows[2]);

    let hint = Paragraph::new(Line::from(Span::styled(
        format!(
            "Path: {} | e ENV | i IDES | arrows move | PgUp/PgDn | Home/End | q/Esc",
            state.scan_path
        ),
        Style::default().fg(Color::DarkGray),
    )))
    .wrap(Wrap { trim: true });
    frame.render_widget(hint, rows[3]);
}

fn render_body(frame: &mut Frame, state: &AppState, area: Rect) {
    let (items, selected, scroll, title) = match state.tab {
        Tab::Env => (
            &state.env.items,
            state.env.selected(),
            state.env.scroll(),
            ".env",
        ),
        Tab::Ides => (
            &state.ides.items,
            state.ides.selected(),
            state.ides.scroll(),
            "IDES",
        ),
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .title(Span::styled(title, Style::default().fg(Color::DarkGray)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let view_h = usize::from(inner.height);
    let start = scroll.min(items.len());
    let end = (start + view_h).min(items.len());
    let visible = &items[start..end];

    let list_items: Vec<ListItem> = visible
        .iter()
        .map(|m| ListItem::new(render_match_line(m)))
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    if !visible.is_empty() {
        let in_view = selected.saturating_sub(start).min(visible.len() - 1);
        list_state.select(Some(in_view));
    }

    let list = List::new(list_items)
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("  ");

    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_footer(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_list();
    let done = active.done;

    let status = if active.is_empty() {
        if done {
            "DONE (0 results)".to_string()
        } else {
            "SCANNING...".to_string()
        }
    } else {
        format!(
            "Sel {}/{} | Scroll {} | {}",
            active.selected() + 1,
            active.len(),
            active.scroll() + 1,
            if done { "DONE" } else { "SCANNING" }
        )
    };

    let style = if done {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let p = Paragraph::new(Line::from(Span::styled(status, style)));
    frame.render_widget(p, area);
}

fn render_match_line(m: &vault_core::KeyMatch) -> Line<'static> {
    let provider_style = provider_style(&m.provider);
    let provider = Span::styled(format!("[{}]", m.provider), provider_style);

    let hardcoded_label = if m.hardcoded {
        "HARDCODEADA"
    } else {
        "posible referencia"
    };

    let rest = format!(
        " {} : L{} -> {} [{}]",
        m.file_path.display(),
        m.line_number,
        m.key,
        hardcoded_label
    );
    Line::from(vec![provider, Span::raw(rest)])
}

fn provider_style(provider: &str) -> Style {
    let p = provider.to_ascii_lowercase();
    if p.contains("openai") {
        return Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    }
    if p.contains("gemini") {
        return Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD);
    }
    Style::default().fg(Color::DarkGray)
}

fn spinner_ascii(tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[(tick as usize) % FRAMES.len()]
}
