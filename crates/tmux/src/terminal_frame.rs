use std::io::Write;

use crate::pane_layout::{PaneRect, PaneSeparator};

pub(crate) struct PaneRender<'a> {
    pub(crate) index: u32,
    pub(crate) rect: PaneRect,
    pub(crate) screen: &'a vt100::Screen,
    pub(crate) active: bool,
}

pub(crate) struct RenderedTerminal {
    pub(crate) frame: Vec<u8>,
    pub(crate) captures: Vec<(u32, Vec<u8>)>,
}

pub(crate) fn render_terminal(
    panes: &[PaneRender<'_>],
    separators: &[PaneSeparator],
) -> RenderedTerminal {
    let mut frame = Vec::new();
    frame.extend_from_slice(b"\x1b[?25l\x1b[0m\x1b[H\x1b[2J");
    for pane in panes {
        render_pane(&mut frame, pane);
    }
    render_separators(&mut frame, separators);
    restore_active_cursor(&mut frame, panes);
    let captures = panes
        .iter()
        .map(|pane| (pane.index, pane.screen.contents().into_bytes()))
        .collect();
    RenderedTerminal { frame, captures }
}

fn render_pane(frame: &mut Vec<u8>, pane: &PaneRender<'_>) {
    if pane.rect.cols == 0 || pane.rect.rows == 0 {
        return;
    }
    for (row, contents) in pane
        .screen
        .rows_formatted(0, pane.rect.cols)
        .take(pane.rect.rows as usize)
        .enumerate()
    {
        let row = pane.rect.y.saturating_add(row as u16).saturating_add(1);
        let col = pane.rect.x.saturating_add(1);
        let _ = write!(frame, "\x1b[{row};{col}H");
        frame.extend_from_slice(&contents);
        frame.extend_from_slice(b"\x1b[0m");
    }
}

fn render_separators(frame: &mut Vec<u8>, separators: &[PaneSeparator]) {
    frame.extend_from_slice(b"\x1b[90m");
    for separator in separators {
        let rect = separator.rect;
        let symbol = if rect.cols == 1 { b'|' } else { b'-' };
        for row in 0..rect.rows {
            let terminal_row = rect.y.saturating_add(row).saturating_add(1);
            let terminal_col = rect.x.saturating_add(1);
            let _ = write!(frame, "\x1b[{terminal_row};{terminal_col}H");
            for _ in 0..rect.cols {
                frame.push(symbol);
            }
        }
    }
    frame.extend_from_slice(b"\x1b[0m");
}

fn restore_active_cursor(frame: &mut Vec<u8>, panes: &[PaneRender<'_>]) {
    let Some(active) = panes.iter().find(|pane| pane.active) else {
        frame.extend_from_slice(b"\x1b[?25l");
        return;
    };
    let (row, col) = active.screen.cursor_position();
    let row = active
        .rect
        .y
        .saturating_add(row.min(active.rect.rows.saturating_sub(1)))
        .saturating_add(1);
    let col = active
        .rect
        .x
        .saturating_add(col.min(active.rect.cols.saturating_sub(1)))
        .saturating_add(1);
    let _ = write!(frame, "\x1b[{row};{col}H");
    frame.extend_from_slice(&active.screen.input_mode_formatted());
    frame.extend_from_slice(if active.screen.hide_cursor() {
        b"\x1b[?25l"
    } else {
        b"\x1b[?25h"
    });
}
