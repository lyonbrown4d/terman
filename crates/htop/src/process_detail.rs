use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    format::{format_bytes, format_duration},
    metrics::ProcessRow,
};

pub(crate) fn process_detail_lines(row: Option<&ProcessRow>) -> Vec<Line<'static>> {
    let Some(row) = row else {
        return vec![muted_line("No selected process".to_string())];
    };
    vec![
        detail_line("PID", row.pid.as_str()),
        detail_line("PPID", row.parent_pid.as_deref().unwrap_or("-")),
        detail_line("Status", row.status.as_str()),
        detail_line("CPU", format!("{:.1}%", row.cpu).as_str()),
        detail_line("Memory", format_bytes(row.memory).as_str()),
        detail_line("Runtime", format_duration(row.run_time).as_str()),
        detail_line("Command", command_summary(row.command.as_str()).as_str()),
    ]
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

fn command_summary(command: &str) -> String {
    const MAX: usize = 120;
    if command.chars().count() <= MAX {
        command.to_string()
    } else {
        let mut output = command.chars().take(MAX).collect::<String>();
        output.push_str("...");
        output
    }
}
