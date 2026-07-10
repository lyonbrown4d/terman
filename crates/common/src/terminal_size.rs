pub fn terminal_rows_without_status(rows: u16) -> u16 {
    rows.saturating_sub(1).max(1)
}