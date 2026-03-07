mod body;
mod common;
mod footer;
mod header;
mod terminal;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Frame, Size},
};

use crate::state::AppState;

pub use terminal::{TerminalGuard, make_terminal};

pub const HEADER_HEIGHT: u16 = 8;
pub const FOOTER_HEIGHT: u16 = 2;

pub fn viewport_height(size: Size) -> usize {
    usize::from(size.height)
        .saturating_sub(usize::from(HEADER_HEIGHT + FOOTER_HEIGHT))
        .saturating_sub(2)
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

    header::render(frame, state, chunks[0], tick);
    body::render(frame, state, chunks[1]);
    footer::render(frame, state, chunks[2], tick);
}
