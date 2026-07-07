use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::format::meter_fill;

pub(crate) fn meter_line(
    label: &str,
    value: f64,
    max: f64,
    width: usize,
    suffix: String,
) -> Line<'static> {
    let filled = meter_fill(value, max, width);
    let color = meter_color(value, max);
    Line::from(vec![
        Span::styled(format!("{label:<3} ["), Style::default().fg(Color::Cyan)),
        Span::styled(
            "#".repeat(filled),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "-".repeat(width.saturating_sub(filled)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(format!("] {suffix}"), Style::default().fg(Color::White)),
    ])
}

fn meter_color(value: f64, max: f64) -> Color {
    if max <= 0.0 {
        return Color::DarkGray;
    }
    match value / max {
        ratio if ratio >= 0.85 => Color::Red,
        ratio if ratio >= 0.60 => Color::Yellow,
        _ => Color::Green,
    }
}
