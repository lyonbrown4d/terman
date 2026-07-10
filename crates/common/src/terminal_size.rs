pub fn terminal_rows_without_status(rows: u16) -> u16 {
    rows.saturating_sub(1).max(1)
}

pub fn is_terminal_last_row(row: u16, rows: u16) -> bool {
    row == rows.saturating_sub(1)
}

pub fn is_current_terminal_last_row(row: u16) -> bool {
    crossterm::terminal::size()
        .map(|(_, rows)| is_terminal_last_row(row, rows))
        .unwrap_or(false)
}
