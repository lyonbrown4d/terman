use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{format::format_bytes, model::Snapshot};

pub(crate) fn draw_network(frame: &mut Frame<'_>, area: Rect, snapshot: &Snapshot, scroll: usize) {
    let mut lines = vec![title_line("INTERFACES")];
    lines.push(network_total_line(snapshot));
    let interface_rows = interface_limit(area, snapshot.sockets.len());
    let interface_start = interface_start(snapshot, interface_rows, scroll);
    for row in snapshot.networks.iter().skip(interface_start).take(interface_rows) {
        lines.push(plain_line(format!(
            "{:<20} rx/s {:>8}  tx/s {:>8}  total {:>10}/{:>10}",
            row.name,
            format_bytes(row.received),
            format_bytes(row.transmitted),
            format_bytes(row.total_received),
            format_bytes(row.total_transmitted)
        )));
    }
    lines.push(title_line("CONNECTIONS"));
    lines.push(plain_line("Proto  Local                         Remote                        State          PID   Process".to_string()));
    let connections = connection_limit(area, interface_rows);
    let connection_start = scroll.min(snapshot.sockets.len().saturating_sub(connections));
    for row in snapshot.sockets.iter().skip(connection_start).take(connections) {
        lines.push(plain_line(format!(
            "{:<5}  {:<29} {:<29} {:<13} {:<5} {}",
            row.protocol,
            trim(row.local.as_str(), 29),
            trim(row.remote.as_str(), 29),
            trim(row.state.as_str(), 13),
            trim(row.pid.as_str(), 5),
            row.process
        )));
    }
    render_block(frame, area, "Network", lines);
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
        "{:<20} rx/s {:>8}  tx/s {:>8}  total {:>10}/{:>10}",
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

fn trim(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        value.chars().take(max.saturating_sub(3)).collect::<String>() + "..."
    }
}

fn body_rows(area: Rect) -> usize {
    area.height.saturating_sub(4) as usize
}