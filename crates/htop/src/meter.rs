use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::format::meter_fill;

const LOW_ZONE: f64 = 0.60;
const MID_ZONE: f64 = 0.85;
const FILL: &str = "|";
const EMPTY: &str = " ";

pub(crate) fn meter_line(
    label: &str,
    value: f64,
    max: f64,
    width: usize,
    suffix: String,
) -> Line<'static> {
    Line::from(meter_spans(label, value, max, width, suffix))
}

pub(crate) fn meter_spans(
    label: &str,
    value: f64,
    max: f64,
    width: usize,
    suffix: String,
) -> Vec<Span<'static>> {
    let filled = meter_fill(value, max, width);
    let mut spans = Vec::with_capacity(7);
    spans.push(Span::styled(
        format!("{label:<3} ["),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));
    append_filled_segments(&mut spans, filled, width);
    spans.push(Span::styled(
        EMPTY.repeat(width.saturating_sub(filled)),
        Style::default().fg(Color::DarkGray),
    ));
    spans.push(Span::styled(format!("] {suffix}"), Style::default().fg(Color::White)));
    spans
}

fn append_filled_segments(spans: &mut Vec<Span<'static>>, filled: usize, width: usize) {
    let low_limit = zone_width(width, LOW_ZONE);
    let mid_limit = zone_width(width, MID_ZONE);
    let low = filled.min(low_limit);
    let mid = filled.saturating_sub(low).min(mid_limit.saturating_sub(low_limit));
    let high = filled.saturating_sub(low + mid);
    append_segment(spans, low, Color::Green);
    append_segment(spans, mid, Color::Yellow);
    append_segment(spans, high, Color::Red);
}

fn append_segment(spans: &mut Vec<Span<'static>>, len: usize, color: Color) {
    if len == 0 {
        return;
    }
    spans.push(Span::styled(
        FILL.repeat(len),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ));
}

fn zone_width(width: usize, ratio: f64) -> usize {
    ((width as f64) * ratio).round().clamp(0.0, width as f64) as usize
}
