use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{
    footer::{self, FooterAction},
    mouse::{MouseAction, MouseContext},
    mouse_rows::terminal_area,
    signal_menu,
};

pub(super) fn handle_signal_mouse(
    event: MouseEvent,
    context: &mut MouseContext<'_>,
) -> MouseAction {
    match event.kind {
        MouseEventKind::ScrollUp => {
            if let Some(menu) = context.signal_menu.as_mut() {
                menu.move_cursor(false);
            }
            MouseAction::Handled
        }
        MouseEventKind::ScrollDown => {
            if let Some(menu) = context.signal_menu.as_mut() {
                menu.move_cursor(true);
            }
            MouseAction::Handled
        }
        MouseEventKind::Down(MouseButton::Right) => MouseAction::CancelKill,
        MouseEventKind::Down(MouseButton::Left) => handle_left_click(event, context),
        _ => MouseAction::Handled,
    }
}

fn handle_left_click(event: MouseEvent, context: &mut MouseContext<'_>) -> MouseAction {
    let area = terminal_area();
    let index = context
        .signal_menu
        .as_ref()
        .and_then(|menu| signal_menu::index_at(area, event.column, event.row, menu.cursor()));
    if let Some(index) = index {
        if let Some(menu) = context.signal_menu.as_mut() {
            menu.set_cursor(index);
        }
        return MouseAction::ConfirmKill;
    }
    if event.row != area.height.saturating_sub(1) {
        return MouseAction::Handled;
    }
    match footer::footer_action_at(
        event.column,
        *context.sort,
        *context.sort_inverted,
        *context.tree,
        context.user_filter.selected(),
        context.filter,
        context.search,
        context.refresh_ms,
        context.signal_menu.as_ref().map(|menu| menu.pid()),
    ) {
        Some(FooterAction::ConfirmKill) => MouseAction::ConfirmKill,
        Some(FooterAction::CancelKill) => MouseAction::CancelKill,
        _ => MouseAction::Handled,
    }
}
