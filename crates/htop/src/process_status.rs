use ratatui::text::{Line, Span};

use crate::model::ProcessRow;

#[derive(Default)]
struct StatusCounts {
    running: usize,
    sleeping: usize,
    stopped: usize,
    zombie: usize,
    other: usize,
}

pub(crate) fn status_summary_line(rows: &[ProcessRow]) -> Line<'static> {
    let counts = rows.iter().fold(StatusCounts::default(), |mut counts, row| {
        match normalize_status(row.status.as_str()) {
            "running" => counts.running += 1,
            "sleeping" => counts.sleeping += 1,
            "stopped" => counts.stopped += 1,
            "zombie" => counts.zombie += 1,
            _ => counts.other += 1,
        }
        counts
    });
    Line::from(Span::raw(format!(
        "States: running {}  sleeping {}  stopped {}  zombie {}  other {}",
        counts.running,
        counts.sleeping,
        counts.stopped,
        counts.zombie,
        counts.other
    )))
}

fn normalize_status(status: &str) -> &'static str {
    let lower = status.to_ascii_lowercase();
    if lower.contains("run") || lower.contains("wake") {
        "running"
    } else if lower.contains("sleep") || lower.contains("idle") || lower.contains("park") {
        "sleeping"
    } else if lower.contains("stop") || lower.contains("tracing") {
        "stopped"
    } else if lower.contains("zombie") || lower.contains("dead") {
        "zombie"
    } else {
        "other"
    }
}
