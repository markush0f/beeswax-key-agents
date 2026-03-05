use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::prelude::Rect;

use vault_core::KeyMatch;

use crate::state::{AppAction, AppState};
use crate::ui;

pub struct App;

impl App {
    pub fn run(
        mut state: AppState,
        env_rx: Receiver<KeyMatch>,
        ide_rx: Receiver<KeyMatch>,
    ) -> AppState {
        let _guard = ui::TerminalGuard::enter().expect("terminal init failed");
        let mut terminal = ui::make_terminal().expect("terminal init failed");

        let mut needs_redraw = true;
        let mut last_render = Instant::now() - Duration::from_millis(1000);
        let mut tick: u64 = 0;

        loop {
            let area = terminal.size().unwrap_or(Rect::new(0, 0, 80, 24));
            let viewport_h = ui::viewport_height(area);

            let mut env_disconnected = false;
            loop {
                match env_rx.try_recv() {
                    Ok(m) => {
                        state.push_env(m, viewport_h);
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
                        state.push_ide(m, viewport_h);
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

            if event::poll(Duration::from_millis(30)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(k)) => {
                        if k.kind != KeyEventKind::Press {
                            continue;
                        }
                        match state.handle_key(k, viewport_h) {
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
