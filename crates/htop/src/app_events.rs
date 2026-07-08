use crossterm::event::KeyCode;

use crate::{
    app_input::find_next,
    metrics::Metrics,
    model::{ProcessRow, SortMode},
    sort_menu::{self, SortMenuAction},
};

pub(crate) fn confirm_mouse_kill(metrics: &mut Metrics, kill_target: &mut Option<String>) {
    if let Some(pid) = kill_target.clone() {
        let _ = metrics.kill_process(pid.as_str());
    }
    *kill_target = None;
}

pub(crate) fn handle_kill_input(
    code: KeyCode,
    metrics: &mut Metrics,
    kill_target: &mut Option<String>,
) -> bool {
    let Some(pid) = kill_target.clone() else { return false; };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let _ = metrics.kill_process(pid.as_str());
            *kill_target = None;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => *kill_target = None,
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
    if !*sort_menu_open { return false; }
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
    if !*help_open { return false; }
    if matches!(code, KeyCode::Esc | KeyCode::F(1)) { *help_open = false; }
    true
}

pub(crate) fn handle_search_input(
    code: KeyCode,
    search: &mut String,
    search_input: &mut Option<String>,
    selected: &mut usize,
    processes: &[ProcessRow],
) -> bool {
    let Some(input) = search_input.as_mut() else { return false; };
    match code {
        KeyCode::Enter => {
            *search = input.trim().to_string();
            *selected = find_next(*selected, processes, search.as_str());
            *search_input = None;
        }
        KeyCode::Esc => *search_input = None,
        KeyCode::Backspace => { input.pop(); }
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
    let Some(input) = filter_input.as_mut() else { return false; };
    match code {
        KeyCode::Enter => {
            *filter = input.trim().to_string();
            *filter_input = None;
        }
        KeyCode::Esc => *filter_input = None,
        KeyCode::Backspace => { input.pop(); }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}