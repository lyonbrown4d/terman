use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{format::{format_bytes, format_duration}, model::ProcessRow};

pub(crate) fn process_detail_lines(row: Option<&ProcessRow>) -> Vec<Line<'static>> {
    let Some(row) = row else {
        return vec![muted_line("No selected process".to_string())];
    };
    let mut lines = vec![
        detail_line("PID", row.pid.as_str()),
        detail_line("PPID", row.parent_pid.as_deref().unwrap_or("-")),
        detail_line("Status", row.status.as_str()),
        detail_line("CPU", format!("{:.1}%", row.cpu).as_str()),
        detail_line("Memory", format_bytes(row.memory).as_str()),
        detail_line("Runtime", format_duration(row.run_time).as_str()),
        detail_line("Read", format!("{}/s  total {}", format_bytes(row.read_rate), format_bytes(row.read_total)).as_str()),
        detail_line("Write", format!("{}/s  total {}", format_bytes(row.written_rate), format_bytes(row.written_total)).as_str()),
    ];
    lines.extend(command_lines(row.command.as_str()));
    lines
}

fn command_lines(command: &str) -> Vec<Line<'static>> {
    const WIDTH: usize = 96;
    if command.is_empty() {
        return vec![detail_line("Command", "-")];
    }
    let mut lines = Vec::new();
    let mut chunk = String::new();
    for ch in command.chars() {
        chunk.push(ch);
        if chunk.chars().count() >= WIDTH {
            lines.push(detail_line(command_label(lines.is_empty()), chunk.as_str()));
            chunk.clear();
        }
    }
    if !chunk.is_empty() {
        lines.push(detail_line(command_label(lines.is_empty()), chunk.as_str()));
    }
    lines
}

fn command_label(first: bool) -> &'static str {
    if first { "Command" } else { "" }
}

fn detail_line(label: &'static str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<8}"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(value.to_string()),
    ])
}

fn muted_line(text: String) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(Color::DarkGray)))
}