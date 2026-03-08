//! Terminal lifecycle management via RAII.
//!
//! Initializing and cleaning up a raw-mode terminal requires running paired operations:
//! enable raw mode ↔ disable raw mode, enter alternate screen ↔ leave alternate screen.
//! If the application panics between these pairs, the terminal is left in an unusable state.
//!
//! [`TerminalGuard`] solves this by bundling both enter and exit operations into a single
//! RAII guard. The terminal is restored to its normal state when the guard is dropped,
//! even in the presence of panics or early returns.

use std::io;

use crossterm::{
    cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// RAII guard that owns the raw terminal mode for the duration of the TUI session.
///
/// ## Lifecycle
///
/// - **On construction** ([`enter`](TerminalGuard::enter)): enables raw mode,
///   switches to the alternate screen buffer, and hides the cursor.
/// - **On drop**: restores the cursor, leaves the alternate screen, and disables raw mode.
///
/// Drop errors are silently ignored, as there is nothing meaningful to do with them
/// at teardown time.
pub struct TerminalGuard;

impl TerminalGuard {
    /// Initializes the terminal for TUI rendering.
    ///
    /// Call this before creating the backend with [`make_terminal`]. The returned
    /// guard must be kept alive for the entire duration of the TUI session.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if raw mode or the alternate screen cannot be entered,
    /// which typically indicates a non-TTY stdout (e.g., running inside a pipe).
    pub fn enter() -> io::Result<TerminalGuard> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    /// Restores the terminal to its original state.
    ///
    /// Restores cursor visibility, exits the alternate screen buffer, and disables
    /// raw mode. Errors are silently discarded.
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// Creates and clears a ratatui [`Terminal`] backed by the crossterm backend.
///
/// Must be called after [`TerminalGuard::enter`] so that the alternate screen
/// is already active when the initial `clear()` is performed.
///
/// # Errors
///
/// Returns an [`io::Error`] if the backend cannot be initialized or the terminal
/// cannot be cleared.
pub fn make_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}
