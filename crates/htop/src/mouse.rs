use crossterm::{
    event::{MouseButton, MouseEvent, MouseEventKind},
    terminal,
};
use ratatui::layout::Rect;

use crate::{
    model::{ProcessRow, SortMode},
    process_detail::process_detail_lines,
    render::Tab,
    sort_menu,
};

pub(crate) struct MouseContext<'a> {
    pub(crate) tab: &'a mut Tab,
    pub(crate) sort: &'a mut SortMode,
    pub(crate) sort_menu_open: &'a mut bool,
    pub(crate) sort_cursor: &'a mut SortMode,
    pub(crate) tree: &'a mut bool,
    pub(crate) help_open: &'a mut bool,
    pub(crate) selected: &'a mut usize,
    pub(crate) processes: &'a [ProcessRow],
}

pub(crate) fn handle_mouse(event: MouseEvent, context: MouseContext<'_>) -> bool {
    if *context.help_open {
        if matches!(event.kind, MouseEventKind::Down(MouseButton::Left)) {
            *context.help_open = false;
        }
        return true;
    }

    match event.kind {
        MouseEventKind::ScrollUp => {
            *context.selected = context.selected.saturating_sub(1);
            true
        }
        MouseEventKind::ScrollDown => {
            *context.selected = move_down(*context.selected, context.processes.len());
            true
        }
        MouseEventKind::Down(MouseButton::Left) => click(event.column, event.row, context),
        _ => false,
    }
}

fn click(column: u16, row: u16, mut context: MouseContext<'_>) -> bool {
    if *context.sort_menu_open {
        if let Some(mode) = sort_menu::mode_at(terminal_area(), column, row) {
            *context.sort_cursor = mode;
            *context.sort = mode;
        }
        *context.sort_menu_open = false;
        return true;
    }

    if handle_footer(column, row, &mut context) {
        return true;
    }

    if let Some(tab) = tab_at(column, row) {
        *context.tab = tab;
        return true;
    }

    if let Some(index) = process_at(*context.tab, row, *context.selected, context.processes) {
        *context.selected = index;
        return true;
    }

    false
}

fn handle_footer(column: u16, row: u16, context: &mut MouseContext<'_>) -> bool {
    if row != terminal_area().height.saturating_sub(1) {
        return false;
    }
    match footer_action(column) {
        Some(FooterAction::Help) => *context.help_open = true,
        Some(FooterAction::Tree) => *context.tree = !*context.tree,
        Some(FooterAction::Sort) => {
            *context.sort_cursor = *context.sort;
            *context.sort_menu_open = true;
        }
        None => return false,
    }
    true
}

enum FooterAction {
    Help,
    Tree,
    Sort,
}

fn footer_action(column: u16) -> Option<FooterAction> {
    match column {
        0..=9 => Some(FooterAction::Help),
        43..=53 => Some(FooterAction::Tree),
        54..=68 => Some(FooterAction::Sort),
        _ => None,
    }
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

fn process_at(tab: Tab, row: u16, selected: usize, processes: &[ProcessRow]) -> Option<usize> {
    if tab != Tab::Processes || processes.is_empty() {
        return None;
    }
    let first_process_row = 7u16;
    if row < first_process_row {
        return None;
    }
    let visible = visible_process_rows(selected, processes);
    let offset = row.saturating_sub(first_process_row) as usize;
    if offset >= visible {
        return None;
    }
    Some(visible_start(selected, visible, processes.len()) + offset)
        .filter(|index| *index < processes.len())
}

fn visible_process_rows(selected: usize, processes: &[ProcessRow]) -> usize {
    let rows = terminal_area().height.saturating_sub(5);
    let body_rows = rows.saturating_sub(4) as usize;
    let details = process_detail_lines(processes.get(selected)).len();
    body_rows.saturating_sub(details + 1).max(1)
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