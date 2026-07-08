const OUTER_HEADER_ROWS: u16 = 5;
const OUTER_FOOTER_ROWS: u16 = 1;
const BLOCK_BORDER_ROWS: u16 = 2;
const SUMMARY_ROWS: usize = 10;
const PROCESS_SECTION_ROWS: usize = 2;
const MIN_PROCESS_ROWS: usize = 3;

pub(crate) fn core_rows(body_height: u16, core_count: usize) -> usize {
    if core_count == 0 {
        return 0;
    }
    let mut rows = inner_rows(body_height).saturating_sub(reserved_rows());
    if core_count > rows && rows > 0 {
        rows = rows.saturating_sub(1);
    }
    rows.min(core_count)
}

pub(crate) fn process_rows(body_height: u16, core_count: usize) -> usize {
    let rows = core_rows(body_height, core_count);
    inner_rows(body_height).saturating_sub(SUMMARY_ROWS + rows + overflow_rows(core_count, rows) + PROCESS_SECTION_ROWS)
}

pub(crate) fn process_rows_for_terminal(terminal_height: u16, core_count: usize) -> usize {
    process_rows(content_height(terminal_height), core_count)
}


pub(crate) fn visible_start(selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible || selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total - visible)
    }
}
pub(crate) fn process_start_row(terminal_height: u16, core_count: usize) -> u16 {
    let body = content_height(terminal_height);
    let rows = core_rows(body, core_count);
    OUTER_HEADER_ROWS
        .saturating_add(1)
        .saturating_add((SUMMARY_ROWS + rows + overflow_rows(core_count, rows) + PROCESS_SECTION_ROWS) as u16)
}

fn inner_rows(body_height: u16) -> usize {
    body_height.saturating_sub(BLOCK_BORDER_ROWS) as usize
}

fn reserved_rows() -> usize {
    SUMMARY_ROWS + PROCESS_SECTION_ROWS + MIN_PROCESS_ROWS
}

fn overflow_rows(core_count: usize, core_rows: usize) -> usize {
    usize::from(core_count > core_rows)
}

fn content_height(terminal_height: u16) -> u16 {
    terminal_height.saturating_sub(OUTER_HEADER_ROWS + OUTER_FOOTER_ROWS)
}