use ratatui::text::{Line, Span};

use crate::{meter::meter_spans, model::CpuCore};

const COLUMN_GAP: usize = 2;
const SUFFIX_WIDTH: usize = 6;

pub(crate) fn core_meter_lines(
    cores: &[CpuCore],
    row_limit: usize,
    columns: usize,
    content_width: u16,
) -> Vec<Line<'static>> {
    let columns = columns.max(1);
    let shown = row_limit.saturating_mul(columns).min(cores.len());
    let meter_width = core_meter_width(content_width, columns);
    let mut lines = Vec::with_capacity(row_limit.saturating_add(1));
    for row in 0..row_limit {
        let mut spans = Vec::new();
        for column in 0..columns {
            let index = row.saturating_mul(columns).saturating_add(column);
            let Some(core) = cores.get(index) else {
                break;
            };
            if column > 0 {
                spans.push(Span::raw(" ".repeat(COLUMN_GAP)));
            }
            spans.extend(meter_spans(
                &format!("C{}", core.index),
                core.usage as f64,
                100.0,
                meter_width,
                format!("{:>5.1}%", core.usage),
            ));
        }
        lines.push(Line::from(spans));
    }
    if cores.len() > shown {
        lines.push(Line::from(Span::raw(format!("+{} CPU", cores.len() - shown))));
    }
    lines
}

fn core_meter_width(content_width: u16, columns: usize) -> usize {
    let gaps = columns.saturating_sub(1).saturating_mul(COLUMN_GAP);
    let column_width = usize::from(content_width)
        .saturating_sub(gaps)
        .checked_div(columns.max(1))
        .unwrap_or(1);
    let fixed_width = 3 + 2 + 2 + SUFFIX_WIDTH;
    column_width.saturating_sub(fixed_width).max(1)
}
