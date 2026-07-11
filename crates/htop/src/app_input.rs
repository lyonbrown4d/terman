use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{model::{ProcessRow, SortMode}, render::Tab};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TreeBranchAction {
    Expand,
    Collapse,
}
pub(crate) fn find_next(selected: usize, processes: &[ProcessRow], term: &str) -> usize {
    let term = term.trim().to_lowercase();
    if term.is_empty() || processes.is_empty() {
        return selected;
    }
    for offset in 1..=processes.len() {
        let index = (selected + offset) % processes.len();
        if process_matches_search(&processes[index], term.as_str()) {
            return index;
        }
    }
    selected
}

fn process_matches_search(row: &ProcessRow, term: &str) -> bool {
    row.pid.contains(term)
        || row.user.to_lowercase().contains(term)
        || row.name.to_lowercase().contains(term)
        || row.command.to_lowercase().contains(term)
}

pub(crate) fn adjust_refresh(refresh_ms: &mut u64, code: KeyCode) {
    match code {
        KeyCode::Char('+') | KeyCode::Char('=') => {
            *refresh_ms = refresh_ms.saturating_sub(100).max(100);
        }
        KeyCode::Char('-') => *refresh_ms = (*refresh_ms + 100).min(60_000),
        _ => {}
    }
}

pub(crate) fn quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(10))
}

pub(crate) fn interrupt_key(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('\u{3}') => true,
        KeyCode::Char('c' | 'C') => key.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}

pub(crate) fn help_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::F(1) | KeyCode::Char('h'))
}

pub(crate) fn search_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::F(3))
}

pub(crate) fn filter_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('/') | KeyCode::F(4))
}

pub(crate) fn user_filter_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('u'))
}

pub(crate) fn sort_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('s') | KeyCode::F(6))
}

pub(crate) fn apply_direct_sort(
    tab: Tab,
    code: KeyCode,
    sort: &mut SortMode,
    inverted: &mut bool,
) -> bool {
    let Some(mode) = direct_sort_key(tab, code) else {
        return false;
    };
    *sort = mode;
    *inverted = false;
    true
}

fn direct_sort_key(tab: Tab, code: KeyCode) -> Option<SortMode> {
    let sort = match code {
        KeyCode::Char('P') => SortMode::Cpu,
        KeyCode::Char('M') => SortMode::Memory,
        KeyCode::Char('T') => SortMode::Time,
        KeyCode::Char('N') => SortMode::Pid,
        _ => return None,
    };
    (matches!(tab, Tab::Overview | Tab::Processes) || sort == SortMode::Pid).then_some(sort)
}

pub(crate) fn tree_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('t') | KeyCode::F(5))
}
pub(crate) fn tree_branch_action(code: KeyCode) -> Option<TreeBranchAction> {
    match code {
        KeyCode::Char('+') | KeyCode::Char('=') => Some(TreeBranchAction::Expand),
        KeyCode::Char('-') => Some(TreeBranchAction::Collapse),
        _ => None,
    }
}

pub(crate) fn tree_toggle_all_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('*'))
}

pub(crate) fn follow_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('F'))
}

pub(crate) fn invert_sort_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('I'))
}

pub(crate) fn delay_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('-')
    )
}

pub(crate) fn priority_delta(code: KeyCode) -> Option<i32> {
    match code {
        KeyCode::F(7) => Some(-1),
        KeyCode::F(8) => Some(1),
        _ => None,
    }
}

pub(crate) fn kill_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::F(9))
}

pub(crate) fn navigation_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End
    )
}

pub(crate) fn move_selection(selected: usize, count: usize, code: KeyCode) -> usize {
    if count == 0 {
        return 0;
    }
    match code {
        KeyCode::Up => selected.saturating_sub(1),
        KeyCode::Down => (selected + 1).min(count - 1),
        KeyCode::PageUp => selected.saturating_sub(10),
        KeyCode::PageDown => (selected + 10).min(count - 1),
        KeyCode::Home => 0,
        KeyCode::End => count - 1,
        _ => selected,
    }
}

pub(crate) fn clamp_selection(selected: usize, count: usize) -> usize {
    if count == 0 { 0 } else { selected.min(count - 1) }
}

pub(crate) fn next_tab(tab: Tab, key: &KeyEvent) -> Tab {
    match key.code {
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => tab.previous(),
        KeyCode::Tab | KeyCode::Right => tab.next(),
        KeyCode::BackTab | KeyCode::Left => tab.previous(),
        KeyCode::Char('1') => Tab::Overview,
        KeyCode::Char('2') => Tab::Processes,
        KeyCode::Char('3') => Tab::Io,
        KeyCode::Char('4') => Tab::Network,
        _ => tab,
    }
}
pub(crate) fn tag_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char(' '))
}

pub(crate) fn untag_all_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('U'))
}
