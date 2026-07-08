use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{format::format_bytes, model::Snapshot};

const IO_NAME_START: u16 = 51;

pub(crate) fn draw_io(frame: &mut Frame<'_>, area: Rect, snapshot: &Snapshot, scroll: usize, selected: usize) {
    let mut lines = vec![title_line("PID        READ/s    WRITE/s   TOTAL R   TOTAL W   NAME")];
    let visible = body_rows(area);
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

fn title_line(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
}

fn plain_line(text: String) -> Line<'static> {
    Line::from(Span::raw(text))
}

fn selected_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::Black).bg(Color::Green)))
}

fn body_rows(area: Rect) -> usize {
    area.height.saturating_sub(4) as usize
}
