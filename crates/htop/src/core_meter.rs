use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{format::meter_fill, metrics::CpuCore};

pub(crate) fn core_meter_lines(cores: &[CpuCore], limit: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for core in cores.iter().take(limit) {
        lines.push(core_meter_line(core));
    }
    if cores.len() > limit {
        lines.push(Line::from(Span::raw(format!(
            "... {} more CPU core(s)",
            cores.len() - limit
        ))));
    }
    lines
}

fn core_meter_line(core: &CpuCore) -> Line<'static> {
    let filled = meter_fill(core.usage as f64, 100.0, 24);
    Line::from(vec![
        Span::styled(format!("C{:<2} [", core.index), Style::default().fg(Color::Cyan)),
        Span::styled(
            "#".repeat(filled),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "-".repeat(24usize.saturating_sub(filled)),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(format!("] {:>5.1}%", core.usage), Style::default().fg(Color::White)),
    ])
}
