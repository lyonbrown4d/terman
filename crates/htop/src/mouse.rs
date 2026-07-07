use crossterm::{
    event::{MouseButton, MouseEvent, MouseEventKind},
    terminal,
};
use ratatui::layout::Rect;

use crate::{
    footer::{self, FooterAction},
    model::{ProcessRow, SortMode},
    process_detail::process_detail_lines,
    render::Tab,
    sort_menu,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseAction {
    Ignored,
    Handled,
    Search,
    Filter,
    Kill,
    DelayFaster,
    DelaySlower,
    Quit,
}

pub(crate) struct MouseContext<'a> {
    pub(crate) tab: &'a mut Tab,
    pub(crate) sort: &'a mut SortMode,
    pub(crate) sort_menu_open: &'a mut bool,
    pub(crate) sort_cursor: &'a mut SortMode,
    pub(crate) tree: &'a mut bool,
    pub(crate) help_open: &'a mut bool,
    pub(crate) selected: &'a mut usize,
    pub(crate) detail_scroll: &'a mut usize,
    pub(crate) io_scroll: &'a mut usize,
    pub(crate) network_scroll: &'a mut usize,
    pub(crate) processes: &'a [ProcessRow],
    pub(crate) cpu_core_count: usize,
    pub(crate) filter: &'a str,
    pub(crate) search: &'a str,
    pub(crate) refresh_ms: u64,
}

pub(crate) fn handle_mouse(event: MouseEvent, mut context: MouseContext<'_>) -> MouseAction {
    if *context.help_open {
        if matches!(event.kind, MouseEventKind::Down(MouseButton::Left)) {
            *context.help_open = false;
        }
        return MouseAction::Handled;
    }

    match event.kind {
        MouseEventKind::ScrollUp => {
            if sort_menu_scroll(&mut context, false) {
                return MouseAction::Handled;
            }
            if tab_scroll(&mut context, false) {
                return MouseAction::Handled;
            }
            if detail_at(event.row, &context) {
                *context.detail_scroll = (*context.detail_scroll).saturating_sub(1);
            } else {
                move_selected(context.selected, context.detail_scroll, context.processes.len(), false);
            }
            MouseAction::Handled
        }
        MouseEventKind::ScrollDown => {
            if sort_menu_scroll(&mut context, true) {
                return MouseAction::Handled;
            }
            if tab_scroll(&mut context, false) {
                return MouseAction::Handled;
            }
            if detail_at(event.row, &context) {
                *context.detail_scroll = move_down(*context.detail_scroll, max_detail_scroll(&context));
            } else {
                move_selected(context.selected, context.detail_scroll, context.processes.len(), true);
            }
            MouseAction::Handled
        }
        MouseEventKind::Down(MouseButton::Left) => click(event.column, event.row, context),
        MouseEventKind::Down(MouseButton::Right) => right_click(event.row, context),
        _ => MouseAction::Ignored,
    }
}

fn tab_scroll(context: &mut MouseContext<'_>, forward: bool) -> bool {
    let target = match *context.tab { Tab::Io => &mut *context.io_scroll, Tab::Network => &mut *context.network_scroll, _ => return false };
    *target = if forward { target.saturating_add(1) } else { target.saturating_sub(1) };
    true
}
fn sort_menu_scroll(context: &mut MouseContext<'_>, forward: bool) -> bool {
    if !*context.sort_menu_open {
        return false;
    }
    sort_menu::move_cursor(context.sort_cursor, forward);
    true
}
fn right_click(row: u16, context: MouseContext<'_>) -> MouseAction {
    let Some(index) = process_at(*context.tab, row, *context.selected, context.processes, context.cpu_core_count) else {
        return MouseAction::Ignored;
    };
    if *context.selected != index {
        *context.detail_scroll = 0;
    }
    *context.selected = index;
    MouseAction::Kill
}
fn click(column: u16, row: u16, mut context: MouseContext<'_>) -> MouseAction {
    if *context.sort_menu_open {
        if let Some(mode) = sort_menu::mode_at(terminal_area(), column, row) {
            *context.sort_cursor = mode;
            *context.sort = mode;
        }
        *context.sort_menu_open = false;
        return MouseAction::Handled;
    }

    match handle_footer(column, row, &mut context) {
        MouseAction::Ignored => {}
        action => return action,
    }

    if let Some(tab) = tab_at(column, row) {
        *context.tab = tab;
        return MouseAction::Handled;
    }

    if let Some(mode) = process_header_sort_at(*context.tab, column, row) {
        *context.sort = mode;
        *context.sort_cursor = mode;
        return MouseAction::Handled;
    }

    if let Some(index) = process_at(*context.tab, row, *context.selected, context.processes, context.cpu_core_count) {
        if *context.selected != index {
            *context.detail_scroll = 0;
        }
        *context.selected = index;
        return MouseAction::Handled;
    }
    MouseAction::Ignored
}

fn handle_footer(column: u16, row: u16, context: &mut MouseContext<'_>) -> MouseAction {
    if row != terminal_area().height.saturating_sub(1) {
        return MouseAction::Ignored;
    }
    match footer::footer_action_at(
        column,
        *context.sort,
        *context.tree,
        context.filter,
        context.search,
        context.refresh_ms,
    ) {
        Some(FooterAction::Help) => *context.help_open = true,
        Some(FooterAction::Search) => return MouseAction::Search,
        Some(FooterAction::Filter) => return MouseAction::Filter,
        Some(FooterAction::Tree) => *context.tree = !*context.tree,
        Some(FooterAction::Sort) => {
            *context.sort_cursor = *context.sort;
            *context.sort_menu_open = true;
        }
        Some(FooterAction::Kill) => return MouseAction::Kill,
        Some(FooterAction::DelayFaster) => return MouseAction::DelayFaster,
        Some(FooterAction::DelaySlower) => return MouseAction::DelaySlower,
        Some(FooterAction::Quit) => return MouseAction::Quit,
        None => return MouseAction::Ignored,
    }
    MouseAction::Handled
}

fn tab_at(column: u16, row: u16) -> Option<Tab> {
    if row != 2 {
        return None;
    }
    let labels = [
        (Tab::Overview, terman_common::builtin_htop_tab_overview_hint()),
        (Tab::Processes, terman_common::builtin_htop_tab_processes_hint()),
        (Tab::Io, terman_common::builtin_htop_tab_io_hint()),
        (Tab::Network, terman_common::builtin_htop_tab_network_hint()),
    ];
    let mut offset = 0u16;
    for (tab, label) in labels {
        let width = label.chars().count() as u16 + 2;
        if column >= offset && column < offset.saturating_add(width) {
            return Some(tab);
        }
        offset = offset.saturating_add(width + 1);
    }
    None
}

fn process_header_sort_at(tab: Tab, column: u16, row: u16) -> Option<SortMode> {
    if tab != Tab::Processes || row != 5 {
        return None;
    }
    match column.saturating_sub(1) {
        0..=10 => Some(SortMode::Pid),
        11..=16 => Some(SortMode::Cpu),
        17..=34 => Some(SortMode::Memory),
        35..=44 => Some(SortMode::Time),
        45..=u16::MAX => Some(SortMode::Name),
    }
}
fn process_at(tab: Tab, row: u16, selected: usize, processes: &[ProcessRow], cores: usize) -> Option<usize> {
    if tab == Tab::Overview { return overview_process_at(row, processes, cores); }
    if tab != Tab::Processes || processes.is_empty() { return None; }
    let first_process_row = 7u16;
    if row < first_process_row { return None; }
    let visible = visible_process_rows(selected, processes);
    let offset = row.saturating_sub(first_process_row) as usize;
    if offset >= visible { return None; }
    Some(visible_start(selected, visible, processes.len()) + offset).filter(|index| *index < processes.len())
}

fn overview_process_at(row: u16, processes: &[ProcessRow], cores: usize) -> Option<usize> {
    let body = terminal_area().height.saturating_sub(5) as usize;
    let core_rows = body.saturating_sub(16).min(cores).min(8);
    let start = 16u16.saturating_add(core_rows as u16);
    let visible = body.saturating_sub(14 + core_rows).min(5);
    row.checked_sub(start).map(usize::from).filter(|index| *index < visible && *index < processes.len())
}

fn visible_process_rows(selected: usize, processes: &[ProcessRow]) -> usize {
    let body_rows = terminal_area().height.saturating_sub(9) as usize;
    let details = process_detail_lines(processes.get(selected)).len();
    body_rows.saturating_sub(detail_rows(details) + 1).max(1)
}

fn detail_at(row: u16, context: &MouseContext<'_>) -> bool {
    if *context.tab != Tab::Processes || context.processes.is_empty() {
        return false;
    }
    let first_detail_row = 7u16.saturating_add(visible_process_rows(*context.selected, context.processes) as u16 + 1);
    let details = process_detail_lines(context.processes.get(*context.selected)).len();
    let end = first_detail_row.saturating_add(detail_rows(details) as u16);
    row >= first_detail_row && row < end
}

fn max_detail_scroll(context: &MouseContext<'_>) -> usize {
    let details = process_detail_lines(context.processes.get(*context.selected)).len();
    details.saturating_sub(detail_rows(details))
}

fn detail_rows(count: usize) -> usize {
    let body_rows = terminal_area().height.saturating_sub(9) as usize;
    let max_detail = body_rows.saturating_sub(4).max(1).min(10);
    count.max(1).min(max_detail)
}

fn move_selected(selected: &mut usize, detail_scroll: &mut usize, count: usize, down: bool) {
    let next = if down { move_down(*selected, count) } else { selected.saturating_sub(1) };
    if next != *selected {
        *detail_scroll = 0;
    }
    *selected = next;
}

fn visible_start(selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible || selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total - visible)
    }
}

fn move_down(selected: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        (selected + 1).min(count - 1)
    }
}

fn terminal_area() -> Rect {
    let (width, height) = terminal::size().unwrap_or((80, 24));
    Rect::new(0, 0, width, height)
}

#[cfg(test)]
mod tests {
    use super::tab_at;
    use crate::render::Tab;

    #[test]
    fn maps_tab_clicks() {
        assert_eq!(tab_at(1, 2), Some(Tab::Overview));
        assert_eq!(tab_at(0, 0), None);
    }
}