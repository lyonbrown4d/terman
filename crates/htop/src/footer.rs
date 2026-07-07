use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::metrics::SortMode;

pub(crate) fn footer_line(
    sort: SortMode,
    tree: bool,
    filter: &str,
    filtering: bool,
    search: &str,
    searching: bool,
    refresh_ms: u64,
    kill_target: Option<&str>,
) -> Line<'static> {
    Line::from(vec![
        key_span("F1"), value_span(" Help ".to_string()),
        key_span("F3"), value_span(format!(" Search:{} ", value_label(search))),
        key_span("F4"), value_span(format!(" Filter:{} ", value_label(filter))),
        key_span("F5"), value_span(format!(" {} ", view_label(tree))),
        key_span("F6"), value_span(format!(" Sort:{} ", sort.label())),
        key_span("F9"), value_span(" Kill ".to_string()),
        key_span("+/-"), value_span(format!(" Delay:{}ms ", refresh_ms)),
        key_span("F10"), value_span(" Quit ".to_string()),
        Span::styled(prompt_text(filtering, searching, kill_target), Style::default().fg(Color::Gray)),
    ])
}

fn key_span(key: &'static str) -> Span<'static> {
    Span::styled(format!(" {key} "), Style::default().fg(Color::Black).bg(Color::Cyan))
}

fn value_span(text: String) -> Span<'static> {
    Span::styled(text, Style::default().fg(Color::White).bg(Color::Blue))
}

fn value_label(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

fn prompt_text(filtering: bool, searching: bool, kill_target: Option<&str>) -> String {
    if let Some(pid) = kill_target {
        format!(" confirm kill pid {pid}: y/n")
    } else if searching {
        " type search, Enter jump, Esc cancel".to_string()
    } else if filtering {
        " type filter, Enter apply, Esc cancel".to_string()
    } else {
        " arrows select, +/- delay".to_string()
    }
}

fn view_label(tree: bool) -> &'static str {
    if tree { "Tree" } else { "Flat" }
}
