use std::collections::HashSet;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    body_layout,
    command_display::ProcessCommandMode,
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
    command_mode: ProcessCommandMode,
    selected: usize,
    filter: &str,
    detail_scroll: usize,
    tagged_pids: &HashSet<String>,
) {
    let details = process_detail_lines(snapshot.processes.get(selected));
    let detail_visible = detail_rows(area, details.len());
    let detail_scroll = detail_scroll.min(details.len().saturating_sub(detail_visible));
    let visible = body_layout::data_rows(area).saturating_sub(detail_visible + 1).max(1);
    let start = visible_start(selected, visible, snapshot.processes.len());
    let mut lines = vec![process_header_line(sort, command_mode)];
    let view = view_label(tree);
    let selection = selection_label(selected, snapshot.processes.len());
    lines.push(plain_line(terman_common::builtin_htop_processes_status_hint(
        sort.label(),
        &view,
        &selection,
        filter_label(filter),
    )));
    for (offset, row) in snapshot.processes.iter().skip(start).take(visible).enumerate() {
        lines.push(process_line(row, start + offset == selected, snapshot.total_memory, area.width.saturating_sub(2), tagged_pids.contains(row.pid.as_str()), command_mode));
    }
    lines.push(title_line(terman_common::builtin_htop_processes_details_hint()));
    lines.extend(details.into_iter().skip(detail_scroll).take(detail_visible));
    render_block(frame, area, terman_common::builtin_htop_processes_title_hint(), lines);
}

fn render_block(frame: &mut Frame<'_>, area: Rect, title: String, lines: Vec<Line<'static>>) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn title_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
}

fn plain_line(text: String) -> Line<'static> {
    Line::from(Span::raw(text))
}

fn filter_label(filter: &str) -> &str {
    if filter.is_empty() { "-" } else { filter }
}

fn view_label(tree: bool) -> String {
    if tree {
        terman_common::builtin_htop_processes_view_tree_hint()
    } else {
        terman_common::builtin_htop_processes_view_flat_hint()
    }
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

fn detail_rows(area: Rect, count: usize) -> usize {
    let max_detail = body_layout::data_rows(area).saturating_sub(4).max(1).min(10);
    count.max(1).min(max_detail)
}