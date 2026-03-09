//! Application state machine for the TUI event loop.
//!
//! This module centralises all mutable state consumed by the rendering layer.
//! By keeping state isolated from the UI code, the render functions receive
//! only immutable references, making the data flow easier to reason about.
//!
//! ## Structure
//!
//! ```text
//! AppState
//! ├── scan_path: String       — Root directory being scanned
//! ├── tab: Tab                — Currently active view (Env | Ides | Files)
//! ├── env: ListState          — Matches from .env* files
//! ├── ides: ListState         — Matches from IDE config dirs
//! └── files: ListState        — Matches from source code (hardcoded only)
//! ```
//!
//! ## Keyboard Routing
//!
//! [`AppState::handle_key`] is the single dispatch point for all keyboard events.
//! It returns an [`AppAction`] telling the event loop whether to keep running or exit.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vault_core::KeyMatch;

/// Selects which scanning domain is currently displayed in the main viewport.
///
/// The active tab determines which [`ListState`] is rendered in the body panel
/// and which receives navigation events (up/down, page, home/end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Matches found in `.env*` configuration files.
    Env,
    /// Matches found in IDE-specific directories (`.vscode`, `.idea`, `.antigravity-server`).
    Ides,
    /// Matches found in project source code files (hardcoded credentials only).
    Files,
}

/// Scrollable list state for a single scanning domain (env, IDE, or files).
///
/// `ListState` manages three related concerns:
///
/// 1. **Storage**: the `items` vector grows asynchronously as the scanner thread
///    emits new matches through its MPSC channel.
/// 2. **Selection**: `selected` tracks which row the user has highlighted.
/// 3. **Viewport scrolling**: `scroll` is the first visible item index. The
///    [`ensure_visible`](ListState::ensure_visible) method keeps `selected` within
///    the visible window as the user navigates.
///
/// ## Tail Following
///
/// When `follow_tail` is `true` (the default), new items auto-scroll the list to
/// keep the most recent match in view — similar to `tail -f`. Navigating up
/// with the keyboard automatically disables tail-following.
#[derive(Default)]
pub struct ListState {
    /// All matches received so far for this scanning domain.
    /// Grows concurrently as the background scanner thread emits results.
    pub items: Vec<KeyMatch>,
    /// `true` once the background scanner thread has finished and its channel is closed.
    pub done: bool,
    /// Index of the currently highlighted row (0-based, clamped to `items.len() - 1`).
    pub(crate) selected: usize,
    /// Index of the first visible row. Used to implement virtual scrolling.
    pub(crate) scroll: usize,
    /// When `true`, new items cause `selected` to advance to the latest entry.
    pub(crate) follow_tail: bool,
}

impl ListState {
    /// Creates a new, empty [`ListState`] with tail-following enabled.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            done: false,
            selected: 0,
            scroll: 0,
            follow_tail: true,
        }
    }

    /// Returns the index of the currently highlighted row.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Returns the index of the first visible row (scroll offset).
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Returns the total number of stored matches.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if no matches have been received yet.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Marks the scanner for this domain as finished.
    ///
    /// Called by the event loop when the MPSC channel becomes `Disconnected`.
    pub fn set_done(&mut self) {
        self.done = true;
    }

    /// Appends a new match and updates the selection when tail-following is active.
    ///
    /// If `follow_tail` is `true`, `selected` is advanced to point at the new item.
    /// After insertion, [`ensure_visible`](Self::ensure_visible) re-adjusts the scroll
    /// offset to keep the selected row in view.
    ///
    /// # Arguments
    ///
    /// * `m` - The newly discovered [`KeyMatch`] to store.
    /// * `viewport_h` - Current visible height of the list widget in rows.
    pub fn push(&mut self, m: KeyMatch, viewport_h: usize) {
        if self.follow_tail {
            self.selected = self.items.len();
        } else if self.items.is_empty() {
            self.selected = 0;
        }

        self.items.push(m);
        self.ensure_visible(viewport_h);
    }

    /// Jumps to the first item and disables tail-following.
    ///
    /// # Arguments
    ///
    /// * `viewport_h` - Current visible height of the list widget in rows.
    pub fn home(&mut self, viewport_h: usize) {
        self.selected = 0;
        self.follow_tail = false;
        self.ensure_visible(viewport_h);
    }

    /// Jumps to the last item and re-enables tail-following.
    ///
    /// # Arguments
    ///
    /// * `viewport_h` - Current visible height of the list widget in rows.
    pub fn end(&mut self, viewport_h: usize) {
        if !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
        self.follow_tail = true;
        self.ensure_visible(viewport_h);
    }

    /// Moves the selection by `delta` rows, clamping at the list boundaries.
    ///
    /// Disables tail-following unless the cursor lands on the very last item.
    ///
    /// # Arguments
    ///
    /// * `delta` - Number of rows to move. Negative values move up.
    /// * `viewport_h` - Current visible height of the list widget in rows.
    pub fn move_selection(&mut self, delta: i32, viewport_h: usize) {
        let len = self.items.len();
        if len == 0 {
            return;
        }

        self.selected = (self.selected as i32 + delta).clamp(0, (len - 1) as i32) as usize;
        self.follow_tail = self.selected + 1 == self.items.len();
        self.ensure_visible(viewport_h);
    }

    /// Moves the selection by `delta_pages` full pages.
    ///
    /// One page equals `viewport_h` rows. Delegates to [`move_selection`](Self::move_selection).
    ///
    /// # Arguments
    ///
    /// * `delta_pages` - Number of pages to move. Negative values page up.
    /// * `viewport_h` - Current visible height of the list widget in rows.
    pub fn page(&mut self, delta_pages: i32, viewport_h: usize) {
        let page = viewport_h.max(1) as i32;
        self.move_selection(delta_pages.saturating_mul(page), viewport_h);
    }

    /// Adjusts the `scroll` offset so that `selected` is always within the visible window.
    ///
    /// Called after every operation that modifies `selected`. Ensures the invariant:
    /// `scroll <= selected < scroll + viewport_h`.
    fn ensure_visible(&mut self, viewport_h: usize) {
        if viewport_h == 0 || self.items.is_empty() {
            self.scroll = 0;
            self.selected = 0;
            return;
        }

        self.selected = self.selected.min(self.items.len() - 1);

        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + viewport_h {
            self.scroll = self.selected + 1 - viewport_h;
        }
    }
}

/// Top-level application state passed to the renderer on every frame.
///
/// `AppState` owns three [`ListState`]s — one per scanning domain — and tracks
/// which [`Tab`] is currently active. It also holds the scan path displayed in the header.
pub struct AppState {
    /// Absolute or relative path that was passed to the scanners.
    pub scan_path: String,
    /// Currently active tab, determining which list is rendered and focused.
    pub tab: Tab,
    /// Matches from `.env*` configuration files.
    pub env: ListState,
    /// Matches from IDE configuration directories.
    pub ides: ListState,
    /// Matches from project source files (hardcoded credentials only).
    pub files: ListState,
}

impl AppState {
    /// Creates a new [`AppState`] with all lists empty and the `Env` tab active.
    ///
    /// # Arguments
    ///
    /// * `scan_path` - The directory path currently being scanned (displayed in the header).
    pub fn new(scan_path: String) -> Self {
        Self {
            scan_path,
            tab: Tab::Env,
            env: ListState::new(),
            ides: ListState::new(),
            files: ListState::new(),
        }
    }

    /// Returns a shared reference to the list for the active tab.
    pub fn active_list(&self) -> &ListState {
        match self.tab {
            Tab::Env => &self.env,
            Tab::Ides => &self.ides,
            Tab::Files => &self.files,
        }
    }

    /// Returns a mutable reference to the list for the active tab.
    pub fn active_list_mut(&mut self) -> &mut ListState {
        match self.tab {
            Tab::Env => &mut self.env,
            Tab::Ides => &mut self.ides,
            Tab::Files => &mut self.files,
        }
    }

    /// Appends a new env match and updates the env list's scroll state.
    pub fn push_env(&mut self, m: KeyMatch, viewport_h: usize) {
        self.env.push(m, viewport_h);
    }

    /// Appends a new IDE match and updates the IDE list's scroll state.
    pub fn push_ide(&mut self, m: KeyMatch, viewport_h: usize) {
        self.ides.push(m, viewport_h);
    }

    /// Appends a new file match and updates the files list's scroll state.
    pub fn push_file(&mut self, m: KeyMatch, viewport_h: usize) {
        self.files.push(m, viewport_h);
    }

    /// Marks the env scanner as finished.
    pub fn set_env_done(&mut self) {
        self.env.set_done();
    }

    /// Marks the IDE scanner as finished.
    pub fn set_ide_done(&mut self) {
        self.ides.set_done();
    }

    /// Marks the files scanner as finished.
    pub fn set_files_done(&mut self) {
        self.files.set_done();
    }

    /// Advances the active tab to the next in the cycle: Env → Files → Ides → Env.
    fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Env => Tab::Files,
            Tab::Files => Tab::Ides,
            Tab::Ides => Tab::Env,
        };
    }

    /// Retreats the active tab to the previous in the cycle: Env → Ides → Files → Env.
    fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Env => Tab::Ides,
            Tab::Files => Tab::Env,
            Tab::Ides => Tab::Files,
        };
    }

    /// Dispatches a keyboard event to the appropriate state mutation.
    ///
    /// ## Key Bindings
    ///
    /// | Key(s) | Action |
    /// |---|---|
    /// | `Ctrl+C`, `Q`, `Esc` | Exit the application |
    /// | `←` / `→` / `Tab` | Switch to the previous / next tab |
    /// | `E` / `I` / `F` | Jump directly to Env / Ides / Files tab |
    /// | `↑` / `↓` | Move selection up / down one row |
    /// | `Page Up` / `Page Down` | Move selection by one full viewport page |
    /// | `Home` / `End` | Jump to the first / last item |
    ///
    /// # Arguments
    ///
    /// * `k` - The crossterm keyboard event to process.
    /// * `viewport_h` - Current visible height of the list widget, used for pagination.
    ///
    /// # Returns
    ///
    /// [`AppAction::Exit`] if the user requested shutdown, [`AppAction::None`] otherwise.
    pub fn handle_key(&mut self, k: KeyEvent, viewport_h: usize) -> AppAction {
        if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
            return AppAction::Exit;
        }

        match k.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => AppAction::Exit,
            KeyCode::Left => {
                self.prev_tab();
                AppAction::None
            }
            KeyCode::Right => {
                self.next_tab();
                AppAction::None
            }
            KeyCode::Tab => {
                self.next_tab();
                AppAction::None
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.tab = Tab::Env;
                AppAction::None
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.tab = Tab::Ides;
                AppAction::None
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.tab = Tab::Files;
                AppAction::None
            }
            KeyCode::Up => {
                self.active_list_mut().move_selection(-1, viewport_h);
                AppAction::None
            }
            KeyCode::Down => {
                self.active_list_mut().move_selection(1, viewport_h);
                AppAction::None
            }
            KeyCode::PageUp => {
                self.active_list_mut().page(-1, viewport_h);
                AppAction::None
            }
            KeyCode::PageDown => {
                self.active_list_mut().page(1, viewport_h);
                AppAction::None
            }
            KeyCode::Home => {
                self.active_list_mut().home(viewport_h);
                AppAction::None
            }
            KeyCode::End => {
                self.active_list_mut().end(viewport_h);
                AppAction::None
            }
            _ => AppAction::None,
        }
    }
}

/// The return value of [`AppState::handle_key`], signalling the event loop's next step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    /// Continue running the event loop — a redraw may or may not be needed.
    None,
    /// Shut down the TUI and return to the terminal.
    Exit,
}
