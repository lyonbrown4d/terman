use ratatui::layout::Rect;

use crate::{
    body_layout,
    model::ProcessRow,
    overview_layout,
    mouse_context::MouseContext,
    process_detail::process_detail_lines,
    render::Tab,
};

pub(crate) fn row_process_at(row: u16, context: &MouseContext<'_>) -> Option<usize> {
    process_at(*context.tab, row, *context.selected, context.processes, context.cpu_core_count)
        .or_else(|| io_process_at(row, context))
        .or_else(|| network_process_at(row, context))
}

pub(crate) fn detail_at(row: u16, context: &MouseContext<'_>) -> bool {
    if *context.tab != Tab::Processes || context.processes.is_empty() {
        return false;
    }
    let details = process_detail_lines(context.processes.get(*context.selected)).len();
    let end = detail_first_row(context).saturating_add(detail_rows(details) as u16);
    row >= detail_first_row(context) && row < end
}

pub(crate) fn detail_drag_scroll(row: u16, context: &MouseContext<'_>) -> usize {
    let details = process_detail_lines(context.processes.get(*context.selected)).len();
    let rows = detail_rows(details).max(1);
    let offset = row.saturating_sub(detail_first_row(context)) as usize;
    let max = max_detail_scroll(context);
    if rows <= 1 { 0 } else { max.saturating_mul(offset) / (rows - 1) }
}

pub(crate) fn max_detail_scroll(context: &MouseContext<'_>) -> usize {
    let details = process_detail_lines(context.processes.get(*context.selected)).len();
    details.saturating_sub(detail_rows(details))
}

pub(crate) fn detail_rows(count: usize) -> usize {
    let body_rows = body_layout::terminal_data_rows();
    let max_detail = body_rows.saturating_sub(4).max(1).min(10);
    count.max(1).min(max_detail)
}

pub(crate) fn move_down(selected: usize, count: usize) -> usize {
    if count == 0 { 0 } else { (selected + 1).min(count - 1) }
}

pub(crate) fn terminal_area() -> Rect {
    body_layout::terminal_area()
}

fn network_process_at(row: u16, context: &MouseContext<'_>) -> Option<usize> {
    if *context.tab != Tab::Network || context.sockets.is_empty() { return None; }
    let data_rows = body_layout::terminal_data_rows();
    let interfaces = body_layout::network_interface_rows(data_rows, context.sockets.len());
    let visible = body_layout::network_connection_rows(data_rows, interfaces);
    let first = body_layout::network_connection_first_row(interfaces);
    let offset = row.checked_sub(first)? as usize;
    if offset >= visible { return None; }
    let start = (*context.network_scroll).min(context.sockets.len().saturating_sub(visible));
    let pid = context.sockets.get(start + offset)?.pid.as_str();
    if pid.is_empty() || pid == "-" { return None; }
    context.processes.iter().position(|process| process.pid == pid)
}

fn io_process_at(row: u16, context: &MouseContext<'_>) -> Option<usize> {
    if *context.tab != Tab::Io || context.io.is_empty() { return None; }
    let first = body_layout::io_first_row();
    if row < first { return None; }
    let visible = body_layout::terminal_data_rows();
    let offset = row.saturating_sub(first) as usize;
    if offset >= visible { return None; }
    let start = (*context.io_scroll).min(context.io.len().saturating_sub(visible));
    let pid = context.io.get(start + offset)?.pid.as_str();
    context.processes.iter().position(|process| process.pid == pid)
}

fn process_at(tab: Tab, row: u16, selected: usize, processes: &[ProcessRow], cores: usize) -> Option<usize> {
    if tab == Tab::Overview { return overview_process_at(row, selected, processes, cores); }
    if tab != Tab::Processes || processes.is_empty() { return None; }
    let first = body_layout::process_first_row();
    if row < first { return None; }
    let visible = visible_process_rows(selected, processes);
    let offset = row.saturating_sub(first) as usize;
    if offset >= visible { return None; }
    Some(visible_start(selected, visible, processes.len()) + offset)
        .filter(|index| *index < processes.len())
}

fn overview_process_at(row: u16, selected: usize, processes: &[ProcessRow], cores: usize) -> Option<usize> {
    let terminal = terminal_area();
    let content_width = terminal.width.saturating_sub(2);
    let first_row = overview_layout::process_start_row(terminal.height, content_width, cores);
    let visible = overview_layout::process_rows_for_terminal(
        terminal.height,
        content_width,
        cores,
    );
    let start = overview_layout::visible_start(selected, visible, processes.len());
    row.checked_sub(first_row)
        .map(|offset| start + usize::from(offset))
        .filter(|index| *index < start + visible && *index < processes.len())
}

fn visible_process_rows(selected: usize, processes: &[ProcessRow]) -> usize {
    let body_rows = body_layout::terminal_data_rows();
    let details = process_detail_lines(processes.get(selected)).len();
    body_rows.saturating_sub(detail_rows(details) + 1).max(1)
}

fn detail_first_row(context: &MouseContext<'_>) -> u16 {
    body_layout::process_first_row()
        .saturating_add(visible_process_rows(*context.selected, context.processes) as u16 + 1)
}

fn visible_start(selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible || selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total - visible)
    }
}