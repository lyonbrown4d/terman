const OUTER_HEADER_ROWS: u16 = 5;
const OUTER_FOOTER_ROWS: u16 = 1;
const BLOCK_BORDER_ROWS: u16 = 2;
const SUMMARY_ROWS: usize = 10;
const PROCESS_SECTION_ROWS: usize = 2;
const MIN_PROCESS_ROWS: usize = 3;

pub(crate) fn core_columns(content_width: u16, core_count: usize) -> usize {
    let available = match content_width {
        144.. => 3,
        72.. => 2,
        _ => 1,
    };
    available.min(core_count.max(1))
}

pub(crate) fn core_rows(
    body_height: u16,
    content_width: u16,
    core_count: usize,
) -> usize {
    if core_count == 0 {
        return 0;
    }
    let columns = core_columns(content_width, core_count);
    let needed = required_core_rows(core_count, columns);
    let mut rows = inner_rows(body_height).saturating_sub(reserved_rows());
    if needed > rows && rows > 0 {
        rows = rows.saturating_sub(1);
    }
    rows.min(needed)
}

pub(crate) fn process_rows(
    body_height: u16,
    content_width: u16,
    core_count: usize,
) -> usize {
    let columns = core_columns(content_width, core_count);
    let rows = core_rows(body_height, content_width, core_count);
    inner_rows(body_height).saturating_sub(
        SUMMARY_ROWS
            + rows
            + overflow_rows(core_count, rows, columns)
            + PROCESS_SECTION_ROWS,
    )
}

pub(crate) fn process_rows_for_terminal(
    terminal_height: u16,
    content_width: u16,
    core_count: usize,
) -> usize {
    process_rows(content_height(terminal_height), content_width, core_count)
}

pub(crate) fn visible_start(selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible || selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total - visible)
    }
}

pub(crate) fn process_start_row(
    terminal_height: u16,
    content_width: u16,
    core_count: usize,
) -> u16 {
    let body = content_height(terminal_height);
    let columns = core_columns(content_width, core_count);
    let rows = core_rows(body, content_width, core_count);
    OUTER_HEADER_ROWS
        .saturating_add(1)
        .saturating_add(
            (SUMMARY_ROWS
                + rows
                + overflow_rows(core_count, rows, columns)
                + PROCESS_SECTION_ROWS) as u16,
        )
}

fn inner_rows(body_height: u16) -> usize {
    body_height.saturating_sub(BLOCK_BORDER_ROWS) as usize
}

fn reserved_rows() -> usize {
    SUMMARY_ROWS + PROCESS_SECTION_ROWS + MIN_PROCESS_ROWS
}

fn required_core_rows(core_count: usize, columns: usize) -> usize {
    core_count.saturating_add(columns.saturating_sub(1)) / columns.max(1)
}

fn overflow_rows(core_count: usize, core_rows: usize, columns: usize) -> usize {
    usize::from(required_core_rows(core_count, columns) > core_rows)
}

fn content_height(terminal_height: u16) -> u16 {
    terminal_height.saturating_sub(OUTER_HEADER_ROWS + OUTER_FOOTER_ROWS)
}
