//! Visual ratatui component hierarchy and layout structures.
//!
//! Submodules here break the application into a standard `header->body->footer` display model.
//! To render, the terminal chunks map exactly to these isolated files for cleaner abstraction.

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

pub const FOOTER_HEIGHT: u16 = 2;

pub fn viewport_height(size: Size) -> usize {
    let header_h = header::preferred_height();
    usize::from(size.height)
        .saturating_sub(usize::from(header_h + FOOTER_HEIGHT))
        .saturating_sub(2)
}

pub fn draw(frame: &mut Frame, state: &AppState, tick: u64) {
    let root = frame.area();
    let header_h = header::preferred_height();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(root);

    header::render(frame, state, chunks[0], tick);
    body::render(frame, state, chunks[1]);
    footer::render(frame, state, chunks[2], tick);
}
