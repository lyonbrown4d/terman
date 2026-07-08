use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    model::{Snapshot, SortMode},
    process_detail::process_detail_lines,
    process_table::{process_header_line, process_line},
};

pub(crate) fn draw_processes(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &Snapshot,
    sort: SortMode,
    tree: bool,
    selected: usize,
    filter: &str,
    detail_scroll: usize,
) {
    let details = process_detail_lines(snapshot.processes.get(selected));
    let detail_visible = detail_rows(area, details.len());
    let detail_scroll = detail_scroll.min(details.len().saturating_sub(detail_visible));
    let visible = body_rows(area).saturating_sub(detail_visible + 1).max(1);
    let start = visible_start(selected, visible, snapshot.processes.len());
    let mut lines = vec![process_header_line(sort)];
    lines.push(plain_line(format!(
        "Sort: {}  View: {}  Sel: {}  Filter: {}",
        sort.label(),
        view_label(tree),
        selection_label(selected, snapshot.processes.len()),
        filter_label(filter)
    )));
    for (offset, row) in snapshot.processes.iter().skip(start).take(visible).enumerate() {
        lines.push(process_line(row, start + offset == selected, snapshot.total_memory, area.width.saturating_sub(2)));
    }
    lines.push(title_line("DETAILS"));
    lines.extend(details.into_iter().skip(detail_scroll).take(detail_visible));
    render_block(frame, area, "Processes", lines);
}

fn render_block(frame: &mut Frame<'_>, area: Rect, title: &'static str, lines: Vec<Line<'static>>) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn title_line(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
}

fn plain_line(text: String) -> Line<'static> {
    Line::from(Span::raw(text))
}

fn filter_label(filter: &str) -> &str {
    if filter.is_empty() { "-" } else { filter }
}

fn view_label(tree: bool) -> &'static str {
    if tree { "Tree" } else { "Flat" }
}

fn selection_label(selected: usize, count: usize) -> String {
    if count == 0 { "0/0".to_string() } else { format!("{}/{}", selected + 1, count) }
}

fn visible_start(selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible || selected < visible {
        0
    } else {
        (selected + 1 - visible).min(total - visible)
    }
}

fn body_rows(area: Rect) -> usize {
    area.height.saturating_sub(4) as usize
}

fn detail_rows(area: Rect, count: usize) -> usize {
    let max_detail = body_rows(area).saturating_sub(4).max(1).min(10);
    count.max(1).min(max_detail)
}