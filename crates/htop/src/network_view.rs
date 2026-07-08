use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{format::format_bytes, model::{Snapshot, SortMode}};

pub(crate) fn draw_network(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &Snapshot,
    sort: SortMode,
    scroll: usize,
    selected: usize,
) {
    let mut lines = vec![title_line("INTERFACES")];
    lines.push(network_total_line(snapshot));
    let interface_rows = interface_limit(area, snapshot.sockets.len());
    let interface_start = interface_start(snapshot, interface_rows, scroll);
    for row in snapshot.networks.iter().skip(interface_start).take(interface_rows) {
        lines.push(plain_line(format!(
            "{} rx/s {:>8}  tx/s {:>8}  total {:>10}/{:>10}",
            fit_cell(row.name.as_str(), 20),
            format_bytes(row.received),
            format_bytes(row.transmitted),
            format_bytes(row.total_received),
            format_bytes(row.total_transmitted)
        )));
    }
    lines.push(title_line("CONNECTIONS"));
    lines.push(connection_header_line(sort));
    let connections = connection_limit(area, interface_rows);
    let selected_pid = snapshot.processes.get(selected).map(|row| row.pid.as_str());
    let connection_start = scroll.min(snapshot.sockets.len().saturating_sub(connections));
    for row in snapshot.sockets.iter().skip(connection_start).take(connections) {
        let text = format!(
            "{}  {} {} {} {} {}",
            fit_cell(row.protocol.as_str(), 5),
            fit_cell(row.local.as_str(), 29),
            fit_cell(row.remote.as_str(), 29),
            fit_cell(row.state.as_str(), 13),
            fit_cell(row.pid.as_str(), 5),
            row.process
        );
        let line = if selected_pid == Some(row.pid.as_str()) { selected_line(text) } else { plain_line(text) };
        lines.push(line);
    }
    render_block(frame, area, "Network", lines);
}

fn connection_header_line(sort: SortMode) -> Line<'static> {
    Line::from(vec![
        header_span("Proto  ".to_string(), false),
        header_span("Local                         ".to_string(), false),
        header_span("Remote                        ".to_string(), false),
        header_span("State          ".to_string(), sort == SortMode::State),
        header_span("PID   ".to_string(), sort == SortMode::Pid),
        header_span("Process".to_string(), sort == SortMode::Name),
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
fn interface_start(snapshot: &Snapshot, visible: usize, scroll: usize) -> usize {
    if snapshot.sockets.is_empty() { scroll.min(snapshot.networks.len().saturating_sub(visible)) } else { 0 }
}

fn network_total_line(snapshot: &Snapshot) -> Line<'static> {
    let rx = snapshot.networks.iter().map(|row| row.received).sum();
    let tx = snapshot.networks.iter().map(|row| row.transmitted).sum();
    let total_rx = snapshot.networks.iter().map(|row| row.total_received).sum();
    let total_tx = snapshot.networks.iter().map(|row| row.total_transmitted).sum();
    plain_line(format!(
        "{} rx/s {:>8}  tx/s {:>8}  total {:>10}/{:>10}",
        "TOTAL",
        format_bytes(rx),
        format_bytes(tx),
        format_bytes(total_rx),
        format_bytes(total_tx)
    ))
}

fn interface_limit(area: Rect, sockets: usize) -> usize {
    if sockets == 0 { body_rows(area).saturating_sub(3) } else { 4.min(body_rows(area).saturating_sub(6)) }
}

fn connection_limit(area: Rect, interfaces: usize) -> usize {
    body_rows(area).saturating_sub(interfaces + 4)
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
fn fit_cell(value: &str, width: usize) -> String {
    terman_common::fit_terminal_text(terman_common::truncate_terminal_text(value, width).as_str(), width)
}

fn body_rows(area: Rect) -> usize {
    area.height.saturating_sub(4) as usize
}

#[cfg(test)]
mod tests {
    use super::fit_cell;

    #[test]
    fn fits_wide_values_by_terminal_width() {
        assert_eq!(fit_cell("服务服务服务服务", 7), "服务...");
    }
}
