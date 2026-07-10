use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{body_layout, format::format_bytes, model::{Snapshot, SortMode}};

const IO_NAME_START: u16 = 51;

pub(crate) fn draw_io(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &Snapshot,
    sort: SortMode,
    scroll: usize,
    selected: usize,
) {
    let mut lines = vec![io_header_line(sort)];
    let visible = body_layout::data_rows(area);
    let selected_pid = snapshot.processes.get(selected).map(|row| row.pid.as_str());
    let start = scroll.min(snapshot.io.len().saturating_sub(visible));
    for row in snapshot.io.iter().skip(start).take(visible) {
        let text = format!(
            "{:<10} {:>9} {:>9} {:>9} {:>9} {}",
            row.pid,
            format_bytes(row.read_rate),
            format_bytes(row.written_rate),
            format_bytes(row.read),
            format_bytes(row.written),
            io_name_cell(row.name.as_str(), area.width.saturating_sub(2))
        );
        let line = if selected_pid == Some(row.pid.as_str()) { selected_line(text) } else { plain_line(text) };
        lines.push(line);
    }
    render_block(frame, area, "I/O", lines);
}

fn io_header_line(sort: SortMode) -> Line<'static> {
    Line::from(vec![
        header_span(format!("{:<10} ", "PID"), sort == SortMode::Pid),
        header_span(format!("{:>9} ", "READ/s"), sort == SortMode::Io),
        header_span(format!("{:>9} ", "WRITE/s"), sort == SortMode::Io),
        header_span(format!("{:>9} ", "TOTAL R"), sort == SortMode::Io),
        header_span(format!("{:>9} ", "TOTAL W"), sort == SortMode::Io),
        header_span("NAME".to_string(), sort == SortMode::Name),
    ])
}

fn header_span(text: String, active: bool) -> Span<'static> {
    let style = if active {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };
    Span::styled(text, style)
}
fn io_name_cell(name: &str, table_width: u16) -> String {
    let width = table_width.saturating_sub(IO_NAME_START).max(1) as usize;
    terman_common::fit_terminal_text(terman_common::truncate_terminal_text(name, width).as_str(), width)
}

fn render_block(frame: &mut Frame<'_>, area: Rect, title: &'static str, lines: Vec<Line<'static>>) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}


fn plain_line(text: String) -> Line<'static> {
    Line::from(Span::raw(text))
}

fn selected_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::Black).bg(Color::Green)))
}
