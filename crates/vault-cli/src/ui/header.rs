use std::sync::OnceLock;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
};

use crate::state::{AppState, Tab};

use super::common::{elide_middle, spinner_ascii};

const HEADER_ART_CONTENT: &str = include_str!("../../../../.vault-header.txt");
const LOGO_MAX_LINES: usize = 6;
const LEFT_MIN_WIDTH: u16 = 56;
const TOP_INFO_LINES: u16 = 8;
const TABS_LINES: u16 = 1;

static HEADER_ART: OnceLock<Vec<String>> = OnceLock::new();

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, tick: u64) {
    let accent = accent_color(state.tab);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(TABS_LINES)])
        .split(inner);

    let logo_width = logo_max_width().min(sections[0].width.saturating_sub(LEFT_MIN_WIDTH));
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(LEFT_MIN_WIDTH),
            Constraint::Length(logo_width),
        ])
        .split(sections[0]);
    let info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // SCAN TARGET
            Constraint::Length(1), // path
            Constraint::Length(1), // spacer
            Constraint::Length(1), // RESULTS
            Constraint::Length(1), // spacer
            Constraint::Length(1), // STATUS
            Constraint::Length(1), // spacer
            Constraint::Length(1), // MODE
        ])
        .split(top[0]);

    if top[1].width > 0 {
        let logo = Paragraph::new(build_logo_lines(accent)).alignment(Alignment::Right);
        frame.render_widget(logo, top[1]);
    }

    render_path_label_line(frame, info[0]);
    render_path_value_line(frame, state, info[1]);
    render_results_line(frame, state, info[3], accent);
    render_status_line(frame, state, info[5]);
    render_mode_line(frame, state, info[7], accent);
    render_bottom_row(frame, state, sections[1], tick, accent);
}

pub fn preferred_height() -> u16 {
    logo_line_count().max(TOP_INFO_LINES) + TABS_LINES + 2
}

fn render_path_label_line(frame: &mut Frame, area: Rect) {
    let line = Paragraph::new(Line::from(vec![Span::styled(
        "SCAN TARGET",
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_path_value_line(frame: &mut Frame, state: &AppState, area: Rect) {
    let scan_path = elide_middle(
        &state.scan_path,
        usize::from(area.width).saturating_sub(1).max(8),
    );
    let line = Paragraph::new(Line::from(vec![Span::styled(
        scan_path,
        Style::default().fg(Color::White),
    )]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_results_line(frame: &mut Frame, state: &AppState, area: Rect, accent: Color) {
    let total = state.env.len() + state.ides.len() + state.files.len();
    let line = Paragraph::new(Line::from(vec![
        Span::styled("RESULTS ", Style::default().fg(Color::DarkGray)),
        Span::styled("ENV ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.env.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   IDES ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.ides.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   FILES ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.files.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   TOTAL ", Style::default().fg(Color::Gray)),
        Span::styled(
            total.to_string(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_status_line(frame: &mut Frame, state: &AppState, area: Rect) {
    let line = Paragraph::new(Line::from(vec![
        Span::styled("STATUS ", Style::default().fg(Color::DarkGray)),
        source_label("ENV"),
        source_chip(state.env.done),
        Span::raw("    "),
        source_label("IDES"),
        source_chip(state.ides.done),
        Span::raw("    "),
        source_label("FILES"),
        source_chip(state.files.done),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_mode_line(frame: &mut Frame, state: &AppState, area: Rect, accent: Color) {
    let line = Paragraph::new(Line::from(vec![
        Span::styled("MODE ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "LIVE STREAM",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("VIEW ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", tab_label(state.tab)),
            Style::default()
                .fg(Color::Black)
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_hotkeys_line(frame: &mut Frame, area: Rect, accent: Color) {
    let line = Paragraph::new(Line::from(vec![
        Span::styled("HOTKEYS ", Style::default().fg(Color::DarkGray)),
        hotkey("E", accent),
        Span::styled(" ENV  ", Style::default().fg(Color::Gray)),
        hotkey("I", accent),
        Span::styled(" IDES  ", Style::default().fg(Color::Gray)),
        hotkey("F", accent),
        Span::styled(" FILES  ", Style::default().fg(Color::Gray)),
        hotkey("TAB", accent),
        Span::styled(" NEXT  ", Style::default().fg(Color::Gray)),
        hotkey("Q", accent),
        Span::styled(" QUIT", Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

fn render_bottom_row(frame: &mut Frame, state: &AppState, area: Rect, tick: u64, accent: Color) {
    let tab_width = tabs_width(state, tick).min(area.width);
    let side_width = hotkeys_width()
        .min(area.width.saturating_sub(tab_width) / 2)
        .max(1);
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(side_width),
            Constraint::Min(1),
            Constraint::Length(side_width),
        ])
        .split(area);

    render_hotkeys_line(frame, row[0], accent);
    render_tabs(frame, state, row[1], tick, accent);
}

fn render_tabs(frame: &mut Frame, state: &AppState, area: Rect, tick: u64, accent: Color) {
    let tab_width = tabs_width(state, tick).min(area.width);
    let tabs_area = Rect {
        x: area.x + area.width.saturating_sub(tab_width) / 2,
        y: area.y,
        width: tab_width,
        height: area.height,
    };
    let tabs = Tabs::new(vec![
        Line::from(tab_title("ENV", state.env.len(), state.env.done, tick)),
        Line::from(tab_title("IDES", state.ides.len(), state.ides.done, tick)),
        Line::from(tab_title(
            "FILES",
            state.files.len(),
            state.files.done,
            tick,
        )),
    ])
    .select(match state.tab {
        Tab::Env => 0,
        Tab::Ides => 1,
        Tab::Files => 2,
    })
    .highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(accent)
            .add_modifier(Modifier::BOLD),
    )
    .style(Style::default().fg(Color::DarkGray))
    .divider("  ");
    frame.render_widget(tabs, tabs_area);
}

fn accent_color(tab: Tab) -> Color {
    match tab {
        Tab::Env => Color::Green,
        Tab::Ides => Color::Magenta,
        Tab::Files => Color::Cyan,
    }
}

fn source_label(label: &str) -> Span<'static> {
    Span::styled(format!("{label} "), Style::default().fg(Color::Gray))
}

fn source_chip(done: bool) -> Span<'static> {
    let (text, color) = if done {
        (" READY ", Color::Green)
    } else {
        (" SCAN ", Color::Yellow)
    };

    Span::styled(
        text,
        Style::default()
            .fg(Color::Black)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn hotkey(key: &str, accent: Color) -> Span<'static> {
    Span::styled(
        format!("[{key}]"),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )
}

fn tab_label(tab: Tab) -> &'static str {
    match tab {
        Tab::Env => "ENV",
        Tab::Ides => "IDES",
        Tab::Files => "FILES",
    }
}

fn tab_title(label: &str, count: usize, done: bool, tick: u64) -> String {
    if done {
        format!("{label} ({count}) DONE")
    } else {
        format!("{label} ({count}) SCAN {}", spinner_ascii(tick))
    }
}

fn tabs_width(state: &AppState, tick: u64) -> u16 {
    let divider = 2;
    let total = tab_title("ENV", state.env.len(), state.env.done, tick)
        .chars()
        .count()
        + tab_title("IDES", state.ides.len(), state.ides.done, tick)
            .chars()
            .count()
        + tab_title("FILES", state.files.len(), state.files.done, tick)
            .chars()
            .count()
        + divider * 2;

    total as u16
}

fn hotkeys_width() -> u16 {
    "HOTKEYS [E] ENV  [I] IDES  [F] FILES  [TAB] NEXT  [Q] QUIT"
        .chars()
        .count() as u16
}

fn build_logo_lines(accent: Color) -> Vec<Line<'static>> {
    load_header_art()
        .iter()
        .take(LOGO_MAX_LINES)
        .cloned()
        .map(|line| {
            Line::from(Span::styled(
                line,
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ))
        })
        .collect()
}

fn logo_max_width() -> u16 {
    load_header_art()
        .iter()
        .take(LOGO_MAX_LINES)
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(0)
}

fn logo_line_count() -> u16 {
    load_header_art().iter().take(LOGO_MAX_LINES).count() as u16
}

fn load_header_art() -> &'static Vec<String> {
    HEADER_ART.get_or_init(|| {
        let parsed = parse_logo_file(HEADER_ART_CONTENT.to_string());
        if !parsed.is_empty() {
            parsed
        } else {
            "Vault".lines().map(|s| s.to_string()).collect()
        }
    })
}

fn parse_logo_file(content: String) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_dimensions() {
        let width = logo_max_width();
        assert!(width > 0);

        let count = logo_line_count();
        assert!(count > 0);
        assert!(count <= LOGO_MAX_LINES as u16);
    }

    #[test]
    fn test_preferred_height() {
        let h = preferred_height();
        // logo_line_count().max(8) + 1 + 2
        assert!(h >= 11);
    }
}
