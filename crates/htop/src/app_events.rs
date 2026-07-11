use crossterm::event::KeyCode;

use crate::{
    app_input::find_next,
    metrics::Metrics,
    model::{ProcessRow, SortMode},
    signal_menu::SignalMenuState,
    sort_menu::{self, SortMenuAction},
};

pub(crate) fn confirm_mouse_signal(
    metrics: &mut Metrics,
    signal_menu: &mut Option<SignalMenuState>,
) {
    send_selected_signal(metrics, signal_menu);
}

pub(crate) fn handle_signal_input(
    code: KeyCode,
    metrics: &mut Metrics,
    signal_menu: &mut Option<SignalMenuState>,
) -> bool {
    let Some(state) = signal_menu.as_mut() else {
        return false;
    };
    match code {
        KeyCode::Up | KeyCode::Char('k') => state.move_cursor(false),
        KeyCode::Down | KeyCode::Char('j') => state.move_cursor(true),
        KeyCode::PageUp => {
            for _ in 0..5 {
                state.move_cursor(false);
            }
        }
        KeyCode::PageDown => {
            for _ in 0..5 {
                state.move_cursor(true);
            }
        }
        KeyCode::Enter | KeyCode::Char('y' | 'Y') => {
            send_selected_signal(metrics, signal_menu);
        }
        KeyCode::Esc | KeyCode::F(9) | KeyCode::Char('n' | 'N') => {
            *signal_menu = None;
        }
        _ => {}
    }
    true
}

pub(crate) fn selected_process_pid(processes: &[ProcessRow], selected: usize) -> Option<String> {
    processes.get(selected).map(|row| row.pid.clone())
}

pub(crate) fn handle_sort_menu_input(
    code: KeyCode,
    sort: &mut SortMode,
    sort_menu_open: &mut bool,
    sort_cursor: &mut SortMode,
) -> bool {
    if !*sort_menu_open {
        return false;
    }
    match sort_menu::handle_key(sort_cursor, code) {
        SortMenuAction::Continue => {}
        SortMenuAction::Apply(selected) => {
            *sort = selected;
            *sort_menu_open = false;
        }
        SortMenuAction::Cancel => *sort_menu_open = false,
    }
    true
}

pub(crate) fn handle_help_input(code: KeyCode, help_open: &mut bool) -> bool {
    if !*help_open {
        return false;
    }
    if matches!(code, KeyCode::Esc | KeyCode::F(1)) {
        *help_open = false;
    }
    true
}

pub(crate) fn handle_search_input(
    code: KeyCode,
    search: &mut String,
    search_input: &mut Option<String>,
    selected: &mut usize,
    processes: &[ProcessRow],
) -> bool {
    let Some(input) = search_input.as_mut() else {
        return false;
    };
    match code {
        KeyCode::Enter => {
            *search = input.trim().to_string();
            *selected = find_next(*selected, processes, search.as_str());
            *search_input = None;
        }
        KeyCode::Esc => *search_input = None,
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}

pub(crate) fn handle_filter_input(
    code: KeyCode,
    filter: &mut String,
    filter_input: &mut Option<String>,
) -> bool {
    let Some(input) = filter_input.as_mut() else {
        return false;
    };
    match code {
        KeyCode::Enter => {
            *filter = input.trim().to_string();
            *filter_input = None;
        }
        KeyCode::Esc => *filter_input = None,
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}

fn send_selected_signal(
    metrics: &mut Metrics,
    signal_menu: &mut Option<SignalMenuState>,
) {
    let Some(state) = signal_menu.take() else {
        return;
    };
    if let Some(signal) = state.selected_signal() {
        let _ = metrics.signal_process(state.pid(), signal);
    }
}