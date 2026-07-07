use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    format::{format_bytes, format_duration},
    model::{ProcessRow, SortMode},
};

pub(crate) fn process_header_line(sort: SortMode) -> Line<'static> {
    Line::from(vec![
        header_span(format!("{:<8}", "PID"), sort == SortMode::Pid),
        header_span(" S ".to_string(), false),
        header_span(format!("{:>5} ", "CPU%"), sort == SortMode::Cpu),
        header_span(format!("{:>5} ", "MEM%"), sort == SortMode::Memory),
        header_span(format!("{:>8} ", "RES"), sort == SortMode::Memory),
        header_span(format!("{:>9} ", "TIME+"), sort == SortMode::Time),
        header_span("COMMAND".to_string(), sort == SortMode::Name),
    ])
}

pub(crate) fn process_line(row: &ProcessRow, selected: bool, total_memory: u64) -> Line<'static> {
    let memory_percent = memory_percent(row.memory, total_memory);
    let state = status_char(row.status.as_str());
    let command = tree_name(row.depth, command_text(row));
    let text = process_text(row, state.as_str(), memory_percent, command.as_str());
    if selected {
        return Line::from(Span::styled(text, selected_style()));
    }
    Line::from(vec![
        Span::styled(format!("{:<8}", row.pid), Style::default().fg(Color::Gray)),
        Span::raw(" "),
        Span::styled(format!("{state:<1}"), status_style(state.as_str())),
        Span::raw(" "),
        Span::styled(format!("{:>5.1} ", row.cpu), usage_style(row.cpu as f64, 100.0)),
        Span::styled(format!("{:>5.1} ", memory_percent), usage_style(memory_percent, 100.0)),
        Span::styled(format!("{:>8} ", format_bytes(row.memory)), Style::default().fg(Color::White)),
        Span::styled(format!("{:>9} ", format_duration(row.run_time)), Style::default().fg(Color::White)),
        Span::raw(command),
    ])
}

fn process_text(row: &ProcessRow, state: &str, memory_percent: f64, command: &str) -> String {
    format!(
        "{:<8} {:<1} {:>5.1} {:>5.1} {:>8} {:>9} {}",
        row.pid,
        state,
        row.cpu,
        memory_percent,
        format_bytes(row.memory),
        format_duration(row.run_time),
        command
    )
}

fn header_span(text: String, active: bool) -> Span<'static> {
    let style = if active {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };
    Span::styled(text, style)
}

fn memory_percent(memory: u64, total_memory: u64) -> f64 {
    if total_memory == 0 { 0.0 } else { memory as f64 * 100.0 / total_memory as f64 }
}

fn command_text(row: &ProcessRow) -> &str {
    if row.command.is_empty() { row.name.as_str() } else { row.command.as_str() }
}

fn tree_name(depth: usize, name: &str) -> String {
    if depth == 0 { name.to_string() } else { format!("{}+- {}", "  ".repeat(depth.min(12)), name) }
}

fn status_char(status: &str) -> String {
    status.chars().next().map(|char| char.to_ascii_uppercase().to_string()).unwrap_or_else(|| "-".to_string())
}

fn status_style(status: &str) -> Style {
    match status {
        "R" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        "S" | "I" => Style::default().fg(Color::Cyan),
        "D" => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        "Z" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "T" => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::DarkGray),
    }
}

fn usage_style(value: f64, max: f64) -> Style {
    if max <= 0.0 {
        return Style::default().fg(Color::DarkGray);
    }
    match value / max {
        ratio if ratio >= 0.85 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ratio if ratio >= 0.60 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Green),
    }
}

fn selected_style() -> Style {
    Style::default().fg(Color::Black).bg(Color::Green)
}