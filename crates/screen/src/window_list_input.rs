use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::mouse_window_list::MouseWindowListState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WindowListKeyAction {
    Noop,
    Redraw,
    Select(usize),
    Cancel,
}

pub(crate) fn handle_window_list_key(
    state: &mut MouseWindowListState,
    key: &KeyEvent,
) -> WindowListKeyAction {
    if key.kind == KeyEventKind::Release {
        return WindowListKeyAction::Noop;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k' | 'K') => {
            state.move_selection(-1);
            WindowListKeyAction::Redraw
        }
        KeyCode::Down | KeyCode::Char('j' | 'J') => {
            state.move_selection(1);
            WindowListKeyAction::Redraw
        }
        KeyCode::PageUp => {
            state.move_selection(-5);
            WindowListKeyAction::Redraw
        }
        KeyCode::PageDown => {
            state.move_selection(5);
            WindowListKeyAction::Redraw
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.select_first();
            WindowListKeyAction::Redraw
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.select_last();
            WindowListKeyAction::Redraw
        }
        KeyCode::Enter | KeyCode::Char(' ') => state.selected_window()
            .map(WindowListKeyAction::Select)
            .unwrap_or(WindowListKeyAction::Cancel),
        KeyCode::Char(value) if value.is_ascii_digit() => {
            let index = value.to_digit(10).unwrap_or_default() as usize;
            if state.select_window(index) {
                WindowListKeyAction::Select(index)
            } else {
                WindowListKeyAction::Noop
            }
        }
        KeyCode::Esc | KeyCode::Char('q' | 'Q' | '"') => WindowListKeyAction::Cancel,
        _ => WindowListKeyAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::{WindowListKeyAction, handle_window_list_key};
    use crate::mouse_window_list::MouseWindowListState;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn moves_and_selects_window() {
        let mut state = MouseWindowListState::default();
        state.sync_windows(vec![0, 2, 4], 0);
        state.set_visible_entries(1, vec![(0, 8), (2, 8), (4, 8)]);
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        assert_eq!(handle_window_list_key(&mut state, &down), WindowListKeyAction::Redraw);
        assert_eq!(state.selected_window(), Some(2));
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(handle_window_list_key(&mut state, &enter), WindowListKeyAction::Select(2));
    }
}