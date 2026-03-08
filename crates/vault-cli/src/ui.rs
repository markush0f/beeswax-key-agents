//! Root UI module — layout composition and global rendering entry point.
//!
//! This module acts as the public gateway to the TUI rendering layer. It owns
//! the top-level `header → body → footer` vertical layout and delegates each
//! section to the corresponding submodule.
//!
//! ## Layout Structure
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │  header  (preferred_height() rows, includes tabs)    │
//! ├──────────────────────────────────────────────────────┤
//! │  body    (expands to fill remaining space)           │
//! ├──────────────────────────────────────────────────────┤
//! │  footer  (FOOTER_HEIGHT = 2 rows)                    │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Viewport Height
//!
//! [`viewport_height`] computes the number of rows available to the body list widget
//! after subtracting the header, footer, and border padding. It is queried by the event
//! loop before each tick so that scroll arithmetic always reflects the live terminal size.

pub(crate) mod body;
mod common;
mod footer;
pub(crate) mod header;
mod terminal;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Frame, Size},
};

use crate::state::AppState;

pub use terminal::{TerminalGuard, make_terminal};

/// Fixed row count reserved for the footer widget (status line + keybinding hint).
pub const FOOTER_HEIGHT: u16 = 2;

/// Computes the number of usable rows inside the body panel for the current terminal size.
///
/// This value is used by [`AppState`](crate::state::AppState) to:
/// - Determine how many items fit on screen for scroll offset calculations.
/// - Power page-up / page-down jumps in [`ListState::page`](crate::state::ListState::page).
///
/// The formula subtracts the header height, the footer height, and 2 rows of border
/// padding (top and bottom border of the body block).
///
/// # Arguments
///
/// * `size` - The current terminal dimensions obtained from `terminal.size()`.
pub(crate) fn viewport_height(size: Size) -> u16 {
    let header_h = header::preferred_height();
    size.height
        .saturating_sub(header_h + FOOTER_HEIGHT)
        .saturating_sub(2)
}

/// Renders the full TUI frame for the given application state and animation tick.
///
/// This function is called once per frame by the [`App`](crate::app::App) event loop.
/// It creates the root vertical layout, splits it into three sections, and delegates
/// rendering to the three submodule `render` functions.
///
/// # Arguments
///
/// * `frame` - Mutable reference to the ratatui `Frame` for the current draw call.
/// * `state` - Immutable reference to the current application state.
/// * `tick` - Monotonically increasing frame counter used by spinner animations.
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
