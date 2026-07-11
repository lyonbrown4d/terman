use unicode_width::UnicodeWidthChar;

pub(crate) fn terminal_history(replay: &[u8], cols: u16, rows: u16) -> Vec<String> {
    let capacity = replay
        .len()
        .saturating_div(usize::from(cols))
        .saturating_add(usize::from(rows))
        .clamp(usize::from(rows), 100_000);
    let mut parser = vt100::Parser::new(rows, cols, capacity);
    parser.process(replay);
    parser.screen_mut().set_scrollback(usize::MAX);
    let scrollback = parser.screen().scrollback();
    let mut lines = Vec::with_capacity(scrollback.saturating_add(usize::from(rows)));
    for position in 0..scrollback {
        parser.screen_mut().set_scrollback(scrollback - position);
        lines.push(row_text(parser.screen(), 0, cols));
    }
    parser.screen_mut().set_scrollback(0);
    for row in 0..rows {
        lines.push(row_text(parser.screen(), row, cols));
    }
    while lines.len() > 1 && lines.first().is_some_and(String::is_empty) {
        lines.remove(0);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn row_text(screen: &vt100::Screen, row: u16, cols: u16) -> String {
    screen.contents_between(row, 0, row, cols).trim_end().to_string()
}

pub(crate) fn char_index_at_column(line: &str, target: usize) -> usize {
    let mut column = 0usize;
    for (index, ch) in line.chars().enumerate() {
        let next = column.saturating_add(ch.width().unwrap_or(0));
        if target < next {
            return index;
        }
        column = next;
    }
    line.chars().count()
}