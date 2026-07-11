pub(crate) use crate::mouse_context::MouseContext;

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{
    body_layout,
    footer::{self, FooterAction},
    model::SortMode,
    mouse_signal::handle_signal_mouse,
    mouse_rows::{detail_at, detail_drag_scroll, max_detail_scroll, move_down, row_process_at,
        terminal_area},
    overview_layout,
    process_table,
    render::Tab,
    sort_menu,
    tab_hitbox::tab_at,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseAction {
    Ignored,
    Handled,
    Search,
    Filter,
    Kill,
    ConfirmKill,
    CancelKill,
    PriorityHigher,
    PriorityLower,
    DelayFaster,
    DelaySlower,
    Quit,
}

pub(crate) fn handle_mouse(event: MouseEvent, mut context: MouseContext<'_>) -> MouseAction {
    if *context.help_open {
        if matches!(
            event.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Down(MouseButton::Middle)
        ) {
            *context.help_open = false;
        }
        return MouseAction::Handled;
    }
    if context.signal_menu.is_some() {
        return handle_signal_mouse(event, &mut context);
    }
    match event.kind {
        MouseEventKind::ScrollUp => scroll(event.row, context, false),
        MouseEventKind::ScrollDown => scroll(event.row, context, true),
        MouseEventKind::ScrollLeft => switch_or_scroll_menu(&mut context, false),
        MouseEventKind::ScrollRight => switch_or_scroll_menu(&mut context, true),
        MouseEventKind::Down(MouseButton::Left) => click(event.column, event.row, context),
        MouseEventKind::Up(MouseButton::Left) => release_click(event.column, event.row, context),
        MouseEventKind::Drag(MouseButton::Left) => drag_select(event.row, context),
        MouseEventKind::Down(MouseButton::Middle) => {
            *context.help_open = true;
            MouseAction::Handled
        }
        MouseEventKind::Down(MouseButton::Right) => right_click(event.row, context),
        _ => MouseAction::Ignored,
    }
}

fn scroll(row: u16, mut context: MouseContext<'_>, down: bool) -> MouseAction {
    if sort_menu_scroll(&mut context, down) || tab_scroll(&mut context, down) {
        return MouseAction::Handled;
    }
    if detail_at(row, &context) {
        *context.detail_scroll = if down {
            move_down(*context.detail_scroll, max_detail_scroll(&context))
        } else {
            (*context.detail_scroll).saturating_sub(1)
        };
    } else {
        move_selected(
            context.selected,
            context.detail_scroll,
            context.processes.len(),
            down,
        );
    }
    MouseAction::Handled
}

fn switch_or_scroll_menu(context: &mut MouseContext<'_>, forward: bool) -> MouseAction {
    if sort_menu_scroll(context, forward) {
        return MouseAction::Handled;
    }
    *context.tab = if forward {
        (*context.tab).next()
    } else {
        (*context.tab).previous()
    };
    MouseAction::Handled
}

fn tab_scroll(context: &mut MouseContext<'_>, forward: bool) -> bool {
    let target = match *context.tab {
        Tab::Io => &mut *context.io_scroll,
        Tab::Network => &mut *context.network_scroll,
        _ => return false,
    };
    *target = if forward {
        target.saturating_add(1)
    } else {
        target.saturating_sub(1)
    };
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
    if *context.sort_menu_open {
        *context.sort_menu_open = false;
        return MouseAction::Handled;
    }
    let Some(index) = row_process_at(row, &context) else {
        return MouseAction::Ignored;
    };
    select_index(index, context);
    MouseAction::Kill
}

fn click(column: u16, row: u16, mut context: MouseContext<'_>) -> MouseAction {
    *context.sort_header_pressed = None;
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
    if let Some(mode) = table_header_sort_at(column, row, &context) {
        apply_header_sort(mode, &mut context);
        *context.sort_header_pressed = Some(mode);
        return MouseAction::Handled;
    }
    if let Some(index) = row_process_at(row, &context) {
        select_index(index, context);
        return MouseAction::Handled;
    }
    MouseAction::Ignored
}

fn release_click(column: u16, row: u16, mut context: MouseContext<'_>) -> MouseAction {
    let pressed = context.sort_header_pressed.take();
    if *context.sort_menu_open {
        return MouseAction::Ignored;
    }
    if let Some(tab) = tab_at(column, row) {
        *context.tab = tab;
        return MouseAction::Handled;
    }
    if let Some(mode) = table_header_sort_at(column, row, &context) {
        if pressed != Some(mode) {
            apply_header_sort(mode, &mut context);
        }
        return MouseAction::Handled;
    }
    let Some(index) = row_process_at(row, &context) else {
        return MouseAction::Ignored;
    };
    select_index(index, context);
    MouseAction::Handled
}

fn drag_select(row: u16, context: MouseContext<'_>) -> MouseAction {
    if detail_at(row, &context) {
        *context.detail_scroll = detail_drag_scroll(row, &context);
        return MouseAction::Handled;
    }
    let Some(index) = row_process_at(row, &context) else {
        return MouseAction::Ignored;
    };
    select_index(index, context);
    MouseAction::Handled
}

fn handle_footer(column: u16, row: u16, context: &mut MouseContext<'_>) -> MouseAction {
    if row != terminal_area().height.saturating_sub(1) {
        return MouseAction::Ignored;
    }
    match footer::footer_action_at(
        column,
        *context.sort,
        *context.sort_inverted,
        *context.tree,
        context.filter,
        context.search,
        context.refresh_ms,
        context.signal_menu.as_ref().map(|menu| menu.pid()),
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
        Some(FooterAction::ConfirmKill) => return MouseAction::ConfirmKill,
        Some(FooterAction::CancelKill) => return MouseAction::CancelKill,
        Some(FooterAction::PriorityHigher) => return MouseAction::PriorityHigher,
        Some(FooterAction::PriorityLower) => return MouseAction::PriorityLower,
        Some(FooterAction::DelayFaster) => return MouseAction::DelayFaster,
        Some(FooterAction::DelaySlower) => return MouseAction::DelaySlower,
        Some(FooterAction::Quit) => return MouseAction::Quit,
        None => return MouseAction::Ignored,
    }
    MouseAction::Handled
}

fn apply_header_sort(mode: SortMode, context: &mut MouseContext<'_>) {
    if *context.sort == mode {
        *context.sort_inverted = !*context.sort_inverted;
    } else {
        *context.sort = mode;
    }
    *context.sort_cursor = mode;
}

fn table_header_sort_at(column: u16, row: u16, context: &MouseContext<'_>) -> Option<SortMode> {
    let column = column.saturating_sub(1);
    match *context.tab {
        Tab::Overview if row == overview_header_row(context) => {
            process_table::sort_at_column(column)
        }
        Tab::Processes if row == 6 => process_table::sort_at_column(column),
        Tab::Io if row == 6 => io_header_sort_at(column),
        Tab::Network if row == network_header_row(context) => network_header_sort_at(column),
        _ => None,
    }
}

fn io_header_sort_at(column: u16) -> Option<SortMode> {
    match column {
        0..=10 => Some(SortMode::Pid),
        11..=50 => Some(SortMode::Io),
        c if c >= 51 => Some(SortMode::Name),
        _ => None,
    }
}

fn network_header_sort_at(column: u16) -> Option<SortMode> {
    match column {
        67..=81 => Some(SortMode::State),
        82..=87 => Some(SortMode::Pid),
        c if c >= 88 => Some(SortMode::Name),
        _ => None,
    }
}

fn overview_header_row(context: &MouseContext<'_>) -> u16 {
    overview_layout::process_start_row(terminal_area().height, context.cpu_core_count)
        .saturating_sub(1)
}

fn network_header_row(context: &MouseContext<'_>) -> u16 {
    let data_rows = body_layout::terminal_data_rows();
    let interfaces = body_layout::network_interface_rows(data_rows, context.sockets.len());
    body_layout::network_header_row(interfaces)
}

fn select_index(index: usize, context: MouseContext<'_>) {
    if *context.selected != index {
        *context.detail_scroll = 0;
    }
    *context.selected = index;
}

fn move_selected(selected: &mut usize, detail_scroll: &mut usize, count: usize, down: bool) {
    let next = if down {
        move_down(*selected, count)
    } else {
        selected.saturating_sub(1)
    };
    if next != *selected {
        *detail_scroll = 0;
    }
    *selected = next;
}