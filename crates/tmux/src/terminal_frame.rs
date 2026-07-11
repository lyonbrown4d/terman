use std::io::Write;

pub(crate) struct RenderedTerminal {
    pub(crate) frame: Vec<u8>,
    pub(crate) capture: Vec<u8>,
}

pub(crate) fn render_terminal(screen: &vt100::Screen) -> RenderedTerminal {
    let (rows, cols) = screen.size();
    let mut frame = Vec::with_capacity(rows as usize * cols as usize + 128);
    frame.extend_from_slice(b"\x1b[?25l\x1b[0m\x1b[H\x1b[2J");

    for (row, contents) in screen.rows_formatted(0, cols).enumerate() {
        let _ = write!(frame, "\x1b[{};1H", row + 1);
        frame.extend_from_slice(&contents);
        frame.extend_from_slice(b"\x1b[0m");
    }

    let (cursor_row, cursor_col) = screen.cursor_position();
    let cursor_row = cursor_row.min(rows.saturating_sub(1)) + 1;
    let cursor_col = cursor_col.min(cols.saturating_sub(1)) + 1;
    let _ = write!(frame, "\x1b[{cursor_row};{cursor_col}H");
    frame.extend_from_slice(&screen.input_mode_formatted());
    frame.extend_from_slice(if screen.hide_cursor() {
        b"\x1b[?25l"
    } else {
        b"\x1b[?25h"
    });

    RenderedTerminal {
        frame,
        capture: screen.contents().into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use super::render_terminal;

    #[test]
    fn renders_formatted_frame_and_plain_capture() {
        let mut parser = vt100::Parser::new(2, 8, 16);
        parser.process(b"plain \x1b[31mred\x1b[0m");
        let rendered = render_terminal(parser.screen());

        assert!(rendered.frame.starts_with(b"\x1b[?25l"));
        assert!(rendered.frame.windows(5).any(|bytes| bytes == b"plain"));
        assert_eq!(String::from_utf8(rendered.capture).unwrap(), "plain re\n");
    }
}