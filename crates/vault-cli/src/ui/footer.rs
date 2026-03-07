use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::state::AppState;

use super::common::spinner_ascii;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect, tick: u64) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let active = state.active_list();
    let status = if active.is_empty() {
        if active.done {
            "No findings in current view (DONE)".to_string()
        } else {
            format!("Scanning current view {}", spinner_ascii(tick))
        }
    } else {
        format!(
            "Selected {}/{} | Scroll {} | {} {}",
            active.selected() + 1,
            active.len(),
            active.scroll() + 1,
            if active.done { "DONE" } else { "SCANNING" },
            if active.done { "" } else { spinner_ascii(tick) },
        )
    };

    frame.render_widget(
        Paragraph::new(status).style(Style::default().fg(if active.done {
            Color::Green
        } else {
            Color::DarkGray
        })),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new("Left/Right/TAB switch view | e/i/f | Up/Down select | q quit")
            .style(Style::default().fg(Color::DarkGray)),
        rows[1],
    );
}
