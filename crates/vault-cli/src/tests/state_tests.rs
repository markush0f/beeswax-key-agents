use crate::state::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;
use vault_core::KeyMatch;

fn mock_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

fn mock_match(name: &str) -> KeyMatch {
    KeyMatch {
        file_path: PathBuf::from("test.txt"),
        line_number: 1,
        key: "secret".to_string(),
        provider: name.to_string(),
        hardcoded: false,
    }
}

#[test]
fn test_list_state_navigation() {
    let mut list = ListState::new();
    let h = 5;

    // Empty list
    list.move_selection(1, h);
    assert_eq!(list.selected(), 0);

    // Push items
    list.push(mock_match("p1"), h);
    list.push(mock_match("p2"), h);
    list.push(mock_match("p3"), h);

    assert_eq!(list.selected(), 2); // Follows tail by default
    assert_eq!(list.scroll(), 0);

    list.move_selection(-1, h);
    assert_eq!(list.selected(), 1);
    assert!(!list.follow_tail);

    list.home(h);
    assert_eq!(list.selected(), 0);

    list.end(h);
    assert_eq!(list.selected(), 2);
    assert!(list.follow_tail);
}

#[test]
fn test_list_state_scrolling() {
    let mut list = ListState::new();
    let h = 3; // Viewport height 3

    for i in 0..10 {
        list.push(mock_match(&format!("p{}", i)), h);
    }

    assert_eq!(list.selected(), 9);
    assert_eq!(list.scroll(), 7); // 9 - 3 + 1

    list.move_selection(-5, h); // Select index 4
    assert_eq!(list.selected(), 4);
    assert_eq!(list.scroll(), 4); // Scrolled up to show 4

    list.home(h);
    assert_eq!(list.scroll(), 0);
}

#[test]
fn test_app_state_tabs() {
    let mut state = AppState::new(".".to_string());
    assert_eq!(state.tab, Tab::Env);

    state.handle_key(mock_key(KeyCode::Right), 10);
    assert_eq!(state.tab, Tab::Ides);

    state.handle_key(mock_key(KeyCode::Right), 10);
    assert_eq!(state.tab, Tab::Files);

    state.handle_key(mock_key(KeyCode::Right), 10);
    assert_eq!(state.tab, Tab::Env);

    state.handle_key(mock_key(KeyCode::Char('f')), 10);
    assert_eq!(state.tab, Tab::Files);
}

#[test]
fn test_app_exit_action() {
    let mut state = AppState::new(".".to_string());

    let action = state.handle_key(mock_key(KeyCode::Char('q')), 10);
    assert_eq!(action, AppAction::Exit);

    let ctrl_c = KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    };
    assert_eq!(state.handle_key(ctrl_c, 10), AppAction::Exit);
}
