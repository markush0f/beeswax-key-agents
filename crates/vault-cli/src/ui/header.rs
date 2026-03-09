//! Header panel — logo, scan info, status indicators, and tab bar.
//!
//! The header spans the top of the TUI. Its layout adapts to the available terminal width:
//!
//! ## ASCII Art Logo
//!
//! The logo is loaded once at startup from `.vault-header.txt` (located at the workspace root)
//! via `include_str!`. It is cached in a [`OnceLock`] and rendered in the accent color of the
//! active tab. If the file is empty or missing, `"Vault"` is used as the fallback.
//!
//! ## Dynamic Accent Color
//!
//! The border, logo, tab highlight, and some label styles change color based on the active tab:
//! - `Env` → Green
//! - `Ides` → Magenta
//! - `Files` → Cyan

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

/// Raw content of the ASCII art logo file embedded at compile time.
const HEADER_ART_CONTENT: &str = include_str!("../../../../.vault-header.txt");

/// Maximum number of logo art lines to render in the right column.
/// Limits the logo to 6 rows regardless of how many lines the art file contains.
const LOGO_MAX_LINES: usize = 6;

/// Minimum column width (in chars) reserved for the left info panel.
/// The logo column takes any remaining space on the right.
const LEFT_MIN_WIDTH: u16 = 60;

/// Number of rows in the top info section.
/// Must match the number of `Constraint::Length(1)` entries in the `info` layout.
const TOP_INFO_LINES: u16 = 6;

/// Number of rows reserved at the bottom of the header for the full-width separator
/// and the tab bar / hotkeys row.
const TABS_LINES: u16 = 2;

static HEADER_ART: OnceLock<Vec<String>> = OnceLock::new();

/// Renders the complete header panel including logo, info lines, and tab bar.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to draw into.
/// * `state` - Current application state.
/// * `area` - Rectangular region allocated to the header by the root layout.
/// * `tick` - Animation tick for the spinner in the tab bar.
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
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let logo_width = logo_max_width().min(sections[0].width.saturating_sub(LEFT_MIN_WIDTH));
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(LEFT_MIN_WIDTH),
            Constraint::Length(logo_width),
        ])
        .split(sections[0]);

    // 6 info rows: label, path, sep, results, sep, status
    let info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // SCAN TARGET label
            Constraint::Length(1), // path value
            Constraint::Length(1), // separator line
            Constraint::Length(1), // results counters
            Constraint::Length(1), // separator line
            Constraint::Length(1), // status chips
        ])
        .split(top[0]);

    if top[1].width > 0 {
        let logo = Paragraph::new(build_logo_lines(accent)).alignment(Alignment::Right);
        frame.render_widget(logo, top[1]);
    }

    render_path_label_line(frame, info[0], accent);
    render_path_value_line(frame, state, info[1]);
    render_separator_line(frame, info[2], accent);
    render_results_line(frame, state, info[3], accent);
    render_separator_line(frame, info[4], accent);
    render_status_line(frame, state, info[5]);
    // Full-width separator spanning the entire inner width (not just the left info column)
    render_separator_line(frame, sections[1], accent);
    render_bottom_row(frame, state, sections[2], tick, accent);
}

/// Returns the preferred row height for the header, in terminal rows.
///
/// Computed as `max(logo_line_count, TOP_INFO_LINES) + TABS_LINES + 2` (for borders).
/// Called by [`ui::viewport_height`](crate::ui::viewport_height) to calculate body space.
pub fn preferred_height() -> u16 {
    logo_line_count().max(TOP_INFO_LINES) + TABS_LINES + 2
}

/// Renders the "SCAN TARGET" section label with a small icon prefix.
fn render_path_label_line(frame: &mut Frame, area: Rect, accent: Color) {
    let line = Paragraph::new(Line::from(vec![
        Span::styled("◆ ", Style::default().fg(accent)),
        Span::styled(
            "SCAN TARGET",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

/// Renders the resolved scan path, truncated with middle-elision.
fn render_path_value_line(frame: &mut Frame, state: &AppState, area: Rect) {
    let scan_path = elide_middle(
        &state.scan_path,
        usize::from(area.width).saturating_sub(3).max(8),
    );
    let line = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            scan_path,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

/// Renders a subtle horizontal separator using dimmed box-drawing characters.
fn render_separator_line(frame: &mut Frame, area: Rect, accent: Color) {
    let width = area.width as usize;
    let dash_line: String = "─".repeat(width.saturating_sub(2));
    let line = Paragraph::new(Line::from(Span::styled(
        format!("╶{}╴", dash_line),
        Style::default().fg(Color::Rgb(
            // Dimmed version of the accent for a subtle divider
            50, 50, 60,
        )),
    )));
    let _ = accent;
    frame.render_widget(line, area);
}

/// Renders the per-domain result counters in a compact badge row.
fn render_results_line(frame: &mut Frame, state: &AppState, area: Rect, accent: Color) {
    let total = state.env.len() + state.ides.len() + state.files.len();
    let line = Paragraph::new(Line::from(vec![
        Span::styled("  FINDINGS  ", Style::default().fg(Color::DarkGray)),
        // ENV badge
        Span::styled(
            " ENV ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", state.env.len()),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        // IDES badge
        Span::styled(
            " IDES ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", state.ides.len()),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        // FILES badge
        Span::styled(
            " FILES ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", state.files.len()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  TOTAL  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            total.to_string(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

/// Renders the scanner status row with colored READY/SCAN pill badges per domain.
fn render_status_line(frame: &mut Frame, state: &AppState, area: Rect) {
    let line = Paragraph::new(Line::from(vec![
        Span::styled("  STATUS  ", Style::default().fg(Color::DarkGray)),
        source_label("ENV"),
        source_chip(state.env.done),
        Span::raw("   "),
        source_label("IDES"),
        source_chip(state.ides.done),
        Span::raw("   "),
        source_label("FILES"),
        source_chip(state.files.done),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

/// Renders the hotkeys hint line with bracketed key indicators.
fn render_hotkeys_line(frame: &mut Frame, area: Rect, accent: Color) {
    let line = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        hotkey("E", accent),
        Span::styled(" ENV  ", Style::default().fg(Color::DarkGray)),
        hotkey("I", accent),
        Span::styled(" IDES  ", Style::default().fg(Color::DarkGray)),
        hotkey("F", accent),
        Span::styled(" FILES  ", Style::default().fg(Color::DarkGray)),
        hotkey("TAB", accent),
        Span::styled(" NEXT  ", Style::default().fg(Color::DarkGray)),
        hotkey("Q", accent),
        Span::styled(" QUIT", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(line, area);
}

/// Lays out the bottom tab bar, centering tabs between two hotkey hint strips.
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

/// Renders the centered tab switcher with live count and spinner.
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
    .divider(Span::styled(
        " │ ",
        Style::default().fg(Color::Rgb(50, 50, 70)),
    ));
    frame.render_widget(tabs, tabs_area);
}

/// Maps the active tab to its accent color.
fn accent_color(tab: Tab) -> Color {
    match tab {
        Tab::Env => Color::Green,
        Tab::Ides => Color::Magenta,
        Tab::Files => Color::Cyan,
    }
}

/// Returns a dimmed label span for a scanner domain name.
fn source_label(label: &str) -> Span<'static> {
    Span::styled(format!("{label} "), Style::default().fg(Color::Gray))
}

/// Returns a styled pill badge indicating the scanner's done/running state.
fn source_chip(done: bool) -> Span<'static> {
    let (text, fg, bg) = if done {
        ("✓ DONE ", Color::Black, Color::Green)
    } else {
        ("⟳ SCAN ", Color::Black, Color::Yellow)
    };

    Span::styled(
        text,
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    )
}

/// Returns a styled bracketed key indicator span.
fn hotkey(key: &str, accent: Color) -> Span<'static> {
    Span::styled(
        format!("[{key}]"),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )
}

/// Builds the tab bar title string for a single tab entry.
fn tab_title(label: &str, count: usize, done: bool, tick: u64) -> String {
    if done {
        format!(" {label} ({count}) ✓ ")
    } else {
        format!(" {label} ({count}) {} ", spinner_ascii(tick))
    }
}

/// Measures the total width needed for all three tab titles.
fn tabs_width(state: &AppState, tick: u64) -> u16 {
    let divider = 3; // " │ " = 3 chars
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

/// Measures the width of the static hotkeys hint string.
fn hotkeys_width() -> u16 {
    "[E] ENV  [I] IDES  [F] FILES  [TAB] NEXT  [Q] QUIT"
        .chars()
        .count() as u16
}

/// Builds the logo line vector with per-line accent styling.
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

/// Returns the maximum line width (in columns) of the loaded logo art.
///
/// Used to allocate the right-side logo column in the header layout.
pub(crate) fn logo_max_width() -> u16 {
    load_header_art()
        .iter()
        .take(LOGO_MAX_LINES)
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(0)
}

/// Returns the number of lines in the loaded logo art (capped at [`LOGO_MAX_LINES`]).
///
/// Used by [`preferred_height`] to ensure the header is tall enough to show the full logo.
pub(crate) fn logo_line_count() -> u16 {
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
