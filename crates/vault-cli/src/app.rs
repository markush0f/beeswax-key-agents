//! Application lifecycle controller and main event loop.
//!
//! [`App`] is responsible for:
//!
//! - Initializing and tearing down the raw terminal environment through [`ui::TerminalGuard`].
//! - Draining three MPSC receiver channels (env, IDE, and file matches) on every loop tick.
//! - Routing keyboard events to [`AppState::handle_key`] and delegating redraw decisions.
//! - Enforcing a target frame cap (≈ 30 FPS, minimum 33 ms between draws) to avoid
//!   burning CPU on idle frames while still feeling responsive.
//!
//! ## Event Loop Timing
//!
//! | Timer | Purpose |
//! |---|---|
//! | `poll(30ms)` | Crossterm input poll timeout — limits key latency. |
//! | `33ms` since last render | Minimum time between full redraws (≈ 30 FPS). |
//! | `125ms` forced tick | Forces a redraw even when no events arrive (for animations). |
//! | `10ms` sleep | Prevents the loop from spinning at 100% CPU between events. |

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::prelude::Size;

use vault_core::KeyMatch;

use crate::state::{AppAction, AppState};
use crate::ui;

/// Zero-sized application controller.
///
/// `App` holds no state of its own — all application state lives in [`AppState`].
/// Its only role is to own the terminal lifecycle and drive the event loop.
pub struct App;

impl App {
    /// Enters TUI mode and runs the main event loop until the user signals an exit.
    ///
    /// On entry, the terminal is switched to raw mode and the alternate screen is
    /// activated via [`ui::TerminalGuard`]. On exit (whether by user input or panic),
    /// the guard restores the terminal to its original state.
    ///
    /// ## Channel Draining
    ///
    /// Each loop iteration drains all three receivers non-blockingly using
    /// [`Receiver::try_recv`]. Matches are pushed into the corresponding [`AppState`]
    /// list, keeping the UI always up to date with the latest scanner output.
    ///
    /// When a channel becomes `Disconnected`, the corresponding scanner thread has
    /// finished. The state's `done` flag is set, which the footer and UI widgets use
    /// to display a "DONE" indicator.
    ///
    /// ## Rendering
    ///
    /// The UI is only redrawn when `needs_redraw` is `true` **and** at least 33 ms have
    /// elapsed since the last frame. This decouples the data ingestion rate from the
    /// rendering rate.
    ///
    /// # Arguments
    ///
    /// * `state` - Initial application state (scan path, empty lists).
    /// * `env_rx` - Receiver for matches from the `.env` scanner thread.
    /// * `ide_rx` - Receiver for matches from the IDE config scanner thread.
    /// * `files_rx` - Receiver for matches from the project files scanner thread.
    ///
    /// # Returns
    ///
    /// The final [`AppState`] after the user exits, which the caller (`main`) uses to
    /// print the summary line.
    pub fn run(
        mut state: AppState,
        env_rx: Receiver<KeyMatch>,
        ide_rx: Receiver<KeyMatch>,
        files_rx: Receiver<KeyMatch>,
    ) -> AppState {
        let _guard = ui::TerminalGuard::enter().expect("terminal init failed");
        let mut terminal = ui::make_terminal().expect("terminal init failed");

        let mut needs_redraw = true;
        let mut last_render = Instant::now() - Duration::from_millis(1000);
        let mut tick: u64 = 0;

        loop {
            let size = terminal.size().unwrap_or(Size {
                width: 80,
                height: 24,
            });
            let viewport_h = ui::viewport_height(size);

            let mut env_disconnected = false;
            loop {
                match env_rx.try_recv() {
                    Ok(m) => {
                        state.push_env(m, viewport_h.into());
                        needs_redraw = true;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        env_disconnected = true;
                        break;
                    }
                }
            }
            if env_disconnected && !state.env.done {
                state.set_env_done();
                needs_redraw = true;
            }

            let mut ide_disconnected = false;
            loop {
                match ide_rx.try_recv() {
                    Ok(m) => {
                        state.push_ide(m, viewport_h.into());
                        needs_redraw = true;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        ide_disconnected = true;
                        break;
                    }
                }
            }
            if ide_disconnected && !state.ides.done {
                state.set_ide_done();
                needs_redraw = true;
            }

            let mut files_disconnected = false;
            loop {
                match files_rx.try_recv() {
                    Ok(m) => {
                        state.push_file(m, viewport_h.into());
                        needs_redraw = true;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        files_disconnected = true;
                        break;
                    }
                }
            }
            if files_disconnected && !state.files.done {
                state.set_files_done();
                needs_redraw = true;
            }

            if event::poll(Duration::from_millis(30)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(k)) => {
                        if k.kind != KeyEventKind::Press {
                            continue;
                        }
                        match state.handle_key(k, viewport_h.into()) {
                            AppAction::None => needs_redraw = true,
                            AppAction::Exit => break,
                        }
                    }
                    Ok(Event::Resize(_, _)) => needs_redraw = true,
                    _ => {}
                }
            }

            let now = Instant::now();
            if now.duration_since(last_render) >= Duration::from_millis(125) {
                tick = tick.wrapping_add(1);
                needs_redraw = true;
            }

            if needs_redraw && now.duration_since(last_render) >= Duration::from_millis(33) {
                let _ = terminal.draw(|f| ui::draw(f, &state, tick));
                last_render = now;
                needs_redraw = false;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        drop(terminal);
        drop(_guard);
        state
    }
}
