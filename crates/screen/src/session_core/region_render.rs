use std::io::Write;

use vt100::{Cell, Color};

use super::{
    region_layout::{RegionRect, ScreenRegionLayout, ScreenRegionView},
    window::ScreenWindowState,
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct CellStyle {
    fg: Color,
    bg: Color,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
}

impl From<&Cell> for CellStyle {
    fn from(cell: &Cell) -> Self {
        Self {
            fg: cell.fgcolor(),
            bg: cell.bgcolor(),
            bold: cell.bold(),
            dim: cell.dim(),
            italic: cell.italic(),
            underline: cell.underline(),
            inverse: cell.inverse(),
        }
    }
}

pub(super) fn render_regions(
    layout: &ScreenRegionLayout,
    windows: &[ScreenWindowState],
    rows: u16,
    cols: u16,
) -> Vec<u8> {
    let views = layout.views(rows.max(1), cols.max(1));
    let bordered = views.len() > 1;
    let mut output = b"[?25l[0m[2J[H".to_vec();

    for view in &views {
        let window = windows.iter().find(|window| window.index() == view.window_index);
        if bordered {
            render_border(&mut output, view);
        }
        if let Some(window) = window {
            render_window(&mut output, window, content_rect(view.rect, bordered));
        }
    }

    output.extend_from_slice(b"[0m");
    restore_cursor(&mut output, &views, windows, bordered);
    output
}

fn render_border(output: &mut Vec<u8>, view: &ScreenRegionView) {
    let rect = view.rect;
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    let color = if view.focused { 36 } else { 90 };
    let label = format!(
        " {}{} ",
        if view.focused { "*" } else { "" },
        view.window_index
    );
    move_to(output, rect.y, rect.x);
    let _ = write!(output, "[1;{color}m+");
    let inner = rect.width.saturating_sub(2) as usize;
    let mut top = "-".repeat(inner);
    for (position, ch) in label.chars().take(inner).enumerate() {
        top.replace_range(position..=position, &ch.to_string());
    }
    output.extend_from_slice(top.as_bytes());
    if rect.width > 1 {
        output.push(b'+');
    }

    if rect.width > 1 {
        for row in 1..rect.height.saturating_sub(1) {
            move_to(output, rect.y + row, rect.x);
            output.push(b'|');
            move_to(output, rect.y + row, rect.x + rect.width - 1);
            output.push(b'|');
        }
    }
    if rect.height > 1 {
        move_to(output, rect.y + rect.height - 1, rect.x);
        output.push(b'+');
        output.extend_from_slice("-".repeat(inner).as_bytes());
        if rect.width > 1 {
            output.push(b'+');
        }
    }
    output.extend_from_slice(b"[0m");
}

fn render_window(output: &mut Vec<u8>, window: &ScreenWindowState, rect: RegionRect) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    let screen = window.terminal_screen();
    for row in 0..rect.height {
        move_to(output, rect.y + row, rect.x);
        let mut current_style = None;
        let mut col = 0;
        while col < rect.width {
            let Some(cell) = screen.cell(row, col) else {
                output.push(b' ');
                col += 1;
                continue;
            };
            if cell.is_wide_continuation() {
                col += 1;
                continue;
            }
            let style = CellStyle::from(cell);
            if current_style != Some(style) {
                write_style(output, style);
                current_style = Some(style);
            }
            if cell.is_wide() && col + 1 >= rect.width {
                output.push(b' ');
            } else if cell.has_contents() {
                output.extend_from_slice(cell.contents().as_bytes());
            } else {
                output.push(b' ');
            }
            col += if cell.is_wide() { 2 } else { 1 };
        }
        output.extend_from_slice(b"[0m");
    }
}

fn content_rect(rect: RegionRect, bordered: bool) -> RegionRect {
    if !bordered || rect.width < 3 || rect.height < 3 {
        return rect;
    }
    RegionRect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width - 2,
        height: rect.height - 2,
    }
}

fn restore_cursor(
    output: &mut Vec<u8>,
    views: &[ScreenRegionView],
    windows: &[ScreenWindowState],
    bordered: bool,
) {
    let Some(view) = views.iter().find(|view| view.focused) else {
        output.extend_from_slice(b"[?25h");
        return;
    };
    let Some(window) = windows.iter().find(|window| window.index() == view.window_index) else {
        output.extend_from_slice(b"[?25h");
        return;
    };
    let rect = content_rect(view.rect, bordered);
    if rect.width == 0 || rect.height == 0 {
        output.extend_from_slice(b"[?25h");
        return;
    }
    let (row, col) = window.terminal_screen().cursor_position();
    move_to(
        output,
        rect.y + row.min(rect.height - 1),
        rect.x + col.min(rect.width - 1),
    );
    output.extend_from_slice(b"[?25h");
}

fn write_style(output: &mut Vec<u8>, style: CellStyle) {
    output.extend_from_slice(b"[0");
    if style.bold {
        output.extend_from_slice(b";1");
    }
    if style.dim {
        output.extend_from_slice(b";2");
    }
    if style.italic {
        output.extend_from_slice(b";3");
    }
    if style.underline {
        output.extend_from_slice(b";4");
    }
    if style.inverse {
        output.extend_from_slice(b";7");
    }
    write_color(output, style.fg, true);
    write_color(output, style.bg, false);
    output.push(b'm');
}

fn write_color(output: &mut Vec<u8>, color: Color, foreground: bool) {
    match color {
        Color::Default => {}
        Color::Idx(index) => {
            let prefix = if foreground { 38 } else { 48 };
            let _ = write!(output, ";{prefix};5;{index}");
        }
        Color::Rgb(red, green, blue) => {
            let prefix = if foreground { 38 } else { 48 };
            let _ = write!(output, ";{prefix};2;{red};{green};{blue}");
        }
    }
}

fn move_to(output: &mut Vec<u8>, row: u16, col: u16) {
    let _ = write!(output, "[{};{}H", row.saturating_add(1), col.saturating_add(1));
}