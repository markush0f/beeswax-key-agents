//! Application state machine and rendering layout.
//!
//! State is centralized securely to ensure UI modules read immutable properties
//! while event loops inject new secrets as they are parsed from channels.
//!
//! Handles pagination, active tabs, array sizing, and viewport visibility boundaries.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use vault_core::KeyMatch;

/// Defines which category of secrets is currently displayed in the main viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Env,
    Ides,
    Files,
}

/// Generic state controller used to hold matched secrets within a specific tab.
///
/// It supports features like auto-scroll on new insertions (if on the tail),
/// arbitrary pagination (Page Up vs Down), and clamping to ensure the user does
/// not scroll into the "void".
#[derive(Default)]
pub struct ListState {
    /// A vector of parsed secrets. Continually grows asynchronously.
    pub items: Vec<KeyMatch>,
    /// Flags whether the thread scanning for this category has terminated.
    pub done: bool,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
    pub(crate) follow_tail: bool,
}

impl ListState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            done: false,
            selected: 0,
            scroll: 0,
            follow_tail: true,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn set_done(&mut self) {
        self.done = true;
    }

    pub fn push(&mut self, m: KeyMatch, viewport_h: usize) {
        if self.follow_tail {
            self.selected = self.items.len();
        } else if self.items.is_empty() {
            self.selected = 0;
        }

        self.items.push(m);
        self.ensure_visible(viewport_h);
    }

    pub fn home(&mut self, viewport_h: usize) {
        self.selected = 0;
        self.follow_tail = false;
        self.ensure_visible(viewport_h);
    }

    pub fn end(&mut self, viewport_h: usize) {
        if !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
        self.follow_tail = true;
        self.ensure_visible(viewport_h);
    }

    pub fn move_selection(&mut self, delta: i32, viewport_h: usize) {
        let len = self.items.len();
        if len == 0 {
            return;
        }

        self.selected = (self.selected as i32 + delta).clamp(0, (len - 1) as i32) as usize;
        self.follow_tail = self.selected + 1 == self.items.len();
        self.ensure_visible(viewport_h);
    }

    pub fn page(&mut self, delta_pages: i32, viewport_h: usize) {
        let page = viewport_h.max(1) as i32;
        self.move_selection(delta_pages.saturating_mul(page), viewport_h);
    }

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

/// Represents the overarching architecture state of the visual client at a specific frame.
pub struct AppState {
    /// Top-level path resolving the directory under inspection.
    pub scan_path: String,
    /// Currently displayed view.
    pub tab: Tab,
    /// Parsed instances from configuration files `.env`.
    pub env: ListState,
    /// Parsed instances from specific isolated IDE paths like `.vscode/`.
    pub ides: ListState,
    /// Parsed generic source code files based strictly on matching heuristics.
    pub files: ListState,
}

impl AppState {
    pub fn new(scan_path: String) -> Self {
        Self {
            scan_path,
            tab: Tab::Env,
            env: ListState::new(),
            ides: ListState::new(),
            files: ListState::new(),
        }
    }

    pub fn active_list(&self) -> &ListState {
        match self.tab {
            Tab::Env => &self.env,
            Tab::Ides => &self.ides,
            Tab::Files => &self.files,
        }
    }

    pub fn active_list_mut(&mut self) -> &mut ListState {
        match self.tab {
            Tab::Env => &mut self.env,
            Tab::Ides => &mut self.ides,
            Tab::Files => &mut self.files,
        }
    }

    pub fn push_env(&mut self, m: KeyMatch, viewport_h: usize) {
        self.env.push(m, viewport_h);
    }

    pub fn push_ide(&mut self, m: KeyMatch, viewport_h: usize) {
        self.ides.push(m, viewport_h);
    }

    pub fn push_file(&mut self, m: KeyMatch, viewport_h: usize) {
        self.files.push(m, viewport_h);
    }

    pub fn set_env_done(&mut self) {
        self.env.set_done();
    }

    pub fn set_ide_done(&mut self) {
        self.ides.set_done();
    }

    pub fn set_files_done(&mut self) {
        self.files.set_done();
    }

    fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Env => Tab::Ides,
            Tab::Ides => Tab::Files,
            Tab::Files => Tab::Env,
        };
    }

    fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Env => Tab::Files,
            Tab::Ides => Tab::Env,
            Tab::Files => Tab::Ides,
        };
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    None,
    Exit,
}
