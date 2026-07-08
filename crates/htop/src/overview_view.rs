use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    core_meter::core_meter_lines,
    format::{format_bytes, format_duration},
    meter::meter_line,
    model::Snapshot,
    process_status::status_summary_line,
    process_table::process_line,
};

pub(crate) fn draw_overview(frame: &mut Frame<'_>, area: Rect, snapshot: &Snapshot, selected: usize) {
    let core_rows = overview_core_rows(area, snapshot.cpu_cores.len());
    let mut lines = vec![
        meter_line("CPU", snapshot.cpu_usage as f64, 100.0, 24, format!(
            "{:>5.1}% across {} core(s)", snapshot.cpu_usage, snapshot.cpu_count
        )),
        plain_line(format!("Host: {}  OS: {}", snapshot.system.hostname, snapshot.system.os)),
        plain_line(format!("Kernel: {}  Arch: {}", snapshot.system.kernel, snapshot.system.arch)),
        meter_line("Mem", snapshot.used_memory as f64, snapshot.total_memory as f64, 24, format!(
            "{} / {}", format_bytes(snapshot.used_memory), format_bytes(snapshot.total_memory)
        )),
        meter_line("Swp", snapshot.used_swap as f64, snapshot.total_swap as f64, 24, format!(
            "{} / {}", format_bytes(snapshot.used_swap), format_bytes(snapshot.total_swap)
        )),
        plain_line(format!("Tasks: {} shown / {} total", snapshot.filtered_process_count, snapshot.process_count)),
        status_summary_line(snapshot.processes.as_slice()),
        plain_line(format!(
            "Net: rx {} / tx {} per refresh",
            format_bytes(snapshot.received_per_refresh),
            format_bytes(snapshot.transmitted_per_refresh)
        )),
        plain_line(format!("Uptime: {}", format_duration(snapshot.uptime))),
        plain_line(format!(
            "Load average: {:.2} {:.2} {:.2}",
            snapshot.load_average.one,
            snapshot.load_average.five,
            snapshot.load_average.fifteen
        )),
    ];
    lines.extend(core_meter_lines(snapshot.cpu_cores.as_slice(), core_rows));
    lines.push(title_line("TOP PROCESSES"));
    for (index, row) in snapshot.processes.iter().take(overview_process_rows(area, core_rows, snapshot.cpu_cores.len())).enumerate() {
        lines.push(process_line(row, index == selected, snapshot.total_memory, area.width.saturating_sub(2)));
    }
    render_block(frame, area, "Overview", lines);
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

fn overview_core_rows(area: Rect, count: usize) -> usize {
    (area.height as usize).saturating_sub(16).min(count).min(8)
}

fn overview_process_rows(area: Rect, core_rows: usize, core_count: usize) -> usize {
    let overflow_row = if core_count > core_rows { 1 } else { 0 };
    (area.height as usize).saturating_sub(14 + core_rows + overflow_row)
}