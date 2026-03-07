use std::{fs, sync::OnceLock};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
};

use crate::state::{AppState, Tab};

use super::common::{elide_middle, spinner_ascii};

const HEADER_ART_FILE: &str = ".vault-header.txt";
const LOGO_MAX_LINES: usize = 6;
const LEFT_MIN_WIDTH: u16 = 36;
const LEFT_INFO_LINES: u16 = 6;
static HEADER_ART: OnceLock<Vec<String>> = OnceLock::new();

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, tick: u64) {
    let accent = match state.tab {
        Tab::Env => Color::Green,
        Tab::Ides => Color::Magenta,
        Tab::Files => Color::Cyan,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let logo_width = logo_max_width();
    let right_width = logo_width.min(inner.width.saturating_sub(LEFT_MIN_WIDTH));

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(LEFT_MIN_WIDTH),
            Constraint::Length(right_width),
        ])
        .split(inner);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(cols[0]);

    // The custom ASCII header uses the right region sized to its real width.
    let logo = Paragraph::new(build_logo_lines(accent)).alignment(Alignment::Right);
    frame.render_widget(logo, cols[1]);

    let env_status = if state.env.done {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };
    let ide_status = if state.ides.done {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };
    let files_status = if state.files.done {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };

    let health = Paragraph::new(Line::from(vec![
        Span::styled(" ENV ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.env.done { " READY " } else { " SCAN " },
            env_status,
        ),
        Span::raw("   "),
        Span::styled(" IDES ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.ides.done { " READY " } else { " SCAN " },
            ide_status,
        ),
        Span::raw("   "),
        Span::styled(" FILES ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if state.files.done {
                " READY "
            } else {
                " SCAN "
            },
            files_status,
        ),
        Span::raw("   "),
        Span::styled(
            spinner_ascii(tick),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(health, left[0]);

    let summary = Paragraph::new(Line::from(vec![
        Span::styled("ENV ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.env.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  IDES ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.ides.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  FILES ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.files.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  TOTAL ", Style::default().fg(Color::Gray)),
        Span::styled(
            (state.env.len() + state.ides.len() + state.files.len()).to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(summary, left[1]);

    let scan_path = elide_middle(
        &state.scan_path,
        usize::from(left[2].width).saturating_sub(14).max(8),
    );
    let path_line = Paragraph::new(Line::from(vec![
        Span::styled("SCAN PATH  ", Style::default().fg(Color::DarkGray)),
        Span::styled(scan_path, Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(path_line, left[2]);

    frame.render_widget(Paragraph::new(""), left[3]);

    let tabs = Tabs::new(vec![
        Line::from(format!(
            "  ENV ({})  {} {}  ",
            state.env.len(),
            if state.env.done { "DONE" } else { "SCAN" },
            if state.env.done {
                " "
            } else {
                spinner_ascii(tick)
            },
        )),
        Line::from(format!(
            "  IDES ({}) {} {}  ",
            state.ides.len(),
            if state.ides.done { "DONE" } else { "SCAN" },
            if state.ides.done {
                " "
            } else {
                spinner_ascii(tick)
            },
        )),
        Line::from(format!(
            "  FILES ({}) {} {}  ",
            state.files.len(),
            if state.files.done { "DONE" } else { "SCAN" },
            if state.files.done {
                " "
            } else {
                spinner_ascii(tick)
            },
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
    .divider("   ");
    frame.render_widget(tabs, left[4]);

    let cache_badge = Paragraph::new(Line::from(vec![
        Span::styled(
            " CACHE ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" BLAKE3 ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            " ON ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(cache_badge, left[5]);
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

pub fn preferred_height() -> u16 {
    // +2 for the outer rounded border
    logo_line_count().max(LEFT_INFO_LINES) + 2
}

fn logo_line_count() -> u16 {
    load_header_art().iter().take(LOGO_MAX_LINES).count() as u16
}

fn load_header_art() -> &'static Vec<String> {
    HEADER_ART.get_or_init(|| {
        fs::read_to_string(HEADER_ART_FILE)
            .ok()
            .map(parse_logo_file)
            .filter(|lines| !lines.is_empty())
            .unwrap_or_else(|| "Vault".lines().map(|s| s.to_string()).collect())
    })
}

fn parse_logo_file(content: String) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}
