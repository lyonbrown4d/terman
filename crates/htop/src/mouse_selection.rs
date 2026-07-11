use super::MouseContext;
use crate::mouse_rows::move_down;

pub(super) fn select_index(index: usize, context: MouseContext<'_>) {
    if *context.selected != index {
        *context.detail_scroll = 0;
    }
    *context.selected = index;
}

pub(super) fn move_selected(
    selected: &mut usize,
    detail_scroll: &mut usize,
    count: usize,
    down: bool,
) {
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