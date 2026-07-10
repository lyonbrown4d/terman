use ratatui::layout::Rect;

const HEADER_ROWS: u16 = 5;
const FOOTER_ROWS: u16 = 1;
const VIEW_RESERVED_ROWS: u16 = 4;

pub(crate) fn terminal_area() -> Rect {
    let (width, height) = terman_common::current_terminal_size().unwrap_or((80, 24));
    Rect::new(0, 0, width, height)
}

pub(crate) fn data_rows(area: Rect) -> usize {
    area.height.saturating_sub(VIEW_RESERVED_ROWS) as usize
}

pub(crate) fn terminal_data_rows() -> usize {
    terminal_area()
        .height
        .saturating_sub(HEADER_ROWS + FOOTER_ROWS + VIEW_RESERVED_ROWS) as usize
}

pub(crate) fn process_first_row() -> u16 {
    HEADER_ROWS + 3
}

pub(crate) fn io_first_row() -> u16 {
    HEADER_ROWS + 2
}

pub(crate) fn network_interface_rows(data_rows: usize, sockets: usize) -> usize {
    if sockets == 0 {
        data_rows.saturating_sub(3)
    } else {
        4.min(data_rows.saturating_sub(6))
    }
}

pub(crate) fn network_connection_rows(data_rows: usize, interfaces: usize) -> usize {
    data_rows.saturating_sub(interfaces + 4)
}

pub(crate) fn network_header_row(interfaces: usize) -> u16 {
    HEADER_ROWS
        .saturating_add(4)
        .saturating_add(interfaces.min(u16::MAX as usize) as u16)
}

pub(crate) fn network_connection_first_row(interfaces: usize) -> u16 {
    network_header_row(interfaces).saturating_add(1)
}
