use ratatui::{text::{Line, Span}};

use crate::{meter::meter_line, model::CpuCore};

pub(crate) fn core_meter_lines(cores: &[CpuCore], limit: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for core in cores.iter().take(limit) {
        lines.push(meter_line(
            &format!("C{}", core.index),
            core.usage as f64,
            100.0,
            24,
            format!("{:>5.1}%", core.usage),
        ));
    }
    if cores.len() > limit {
        lines.push(Line::from(Span::raw(format!(
            "... {} more CPU core(s)",
            cores.len() - limit
        ))));
    }
    lines
}
