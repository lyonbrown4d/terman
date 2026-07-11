use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::model::SortMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FooterAction {
    Help,
    Search,
    Filter,
    Tree,
    Sort,
    Kill,
    ConfirmKill,
    CancelKill,
    DelayFaster,
    DelaySlower,
    Quit,
}
pub(crate) fn footer_line(
    sort: SortMode,
    sort_inverted: bool,
    tree: bool,
    filter: &str,
    filtering: bool,
    search: &str,
    searching: bool,
    refresh_ms: u64,
    kill_target: Option<&str>,
) -> Line<'static> {
    let mut spans = vec![
        key_span("F1"), value_span(" Help ".to_string()),
        key_span("F3"), value_span(format!(" Search:{} ", value_label(search))),
        key_span("F4"), value_span(format!(" Filter:{} ", value_label(filter))),
        key_span("F5"), value_span(format!(" {} ", view_label(tree))),
        key_span("F6"), value_span(sort_label(sort, sort_inverted)),
        key_span("F9"), value_span(" Kill ".to_string()),
        key_span("+/-"), value_span(format!(" Delay:{}ms ", refresh_ms)),
        key_span("F10"), value_span(" Quit ".to_string()),
    ];
    spans.extend(prompt_spans(filtering, searching, kill_target));
    Line::from(spans)
}

pub(crate) fn footer_action_at(
    column: u16,
    sort: SortMode,
    sort_inverted: bool,
    tree: bool,
    filter: &str,
    search: &str,
    refresh_ms: u64,
    kill_target: Option<&str>,
) -> Option<FooterAction> {
    let segments = [
        (FooterAction::Help, button_width("F1", " Help ".to_string())),
        (FooterAction::Search, button_width("F3", format!(" Search:{} ", value_label(search)))),
        (FooterAction::Filter, button_width("F4", format!(" Filter:{} ", value_label(filter)))),
        (FooterAction::Tree, button_width("F5", format!(" {} ", view_label(tree)))),
        (FooterAction::Sort, button_width("F6", sort_label(sort, sort_inverted))),
        (FooterAction::Kill, button_width("F9", " Kill ".to_string())),
        (FooterAction::DelayFaster, button_width("+/-", format!(" Delay:{}ms ", refresh_ms))),
        (FooterAction::Quit, button_width("F10", " Quit ".to_string())),
    ];
    let mut start = 0u16;
    if let Some(pid) = kill_target {
        for (_, width) in segments { start = start.saturating_add(width); }
        return kill_prompt_action_at(column.saturating_sub(start), pid);
    }
    for (action, width) in segments {
        let end = start.saturating_add(width);
        if column >= start && column < end {
            if action == FooterAction::DelayFaster && column.saturating_sub(start) >= width / 2 {
                return Some(FooterAction::DelaySlower);
            }
            return Some(action);
        }
        start = end;
    }
    kill_target.and_then(|pid| kill_prompt_action_at(column.saturating_sub(start), pid))
}

fn sort_label(sort: SortMode, inverted: bool) -> String {
    format!(" Sort:{} {} ", sort.label(), sort.direction_label(inverted))
}

fn prompt_spans(filtering: bool, searching: bool, kill_target: Option<&str>) -> Vec<Span<'static>> {
    let Some(pid) = kill_target else { return vec![Span::styled(prompt_text(filtering, searching, None), Style::default().fg(Color::Gray))]; };
    vec![
        Span::styled(terman_common::builtin_htop_signal_footer_hint(pid), Style::default().fg(Color::Gray)),
        key_span("Y"), value_span(" Yes ".to_string()),
        key_span("N"), value_span(" No ".to_string()),
    ]
}

fn kill_prompt_action_at(column: u16, pid: &str) -> Option<FooterAction> {
    let mut start = terman_common::terminal_text_width(&terman_common::builtin_htop_signal_footer_hint(pid));
    let yes = button_width("Y", " Yes ".to_string());
    if column >= start && column < start.saturating_add(yes) { return Some(FooterAction::ConfirmKill); }
    start = start.saturating_add(yes);
    let no = button_width("N", " No ".to_string());
    if column >= start && column < start.saturating_add(no) { Some(FooterAction::CancelKill) } else { None }
}
fn button_width(key: &str, value: String) -> u16 {
    terman_common::terminal_text_width(key).saturating_add(2).saturating_add(terman_common::terminal_text_width(&value))
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
        format!("{}y/n", terman_common::builtin_htop_signal_footer_hint(pid))
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
#[cfg(test)]
mod tests {
    use super::{FooterAction, button_width, footer_action_at};
    use crate::model::SortMode;

    #[test]
    fn maps_footer_actions_after_wide_search_text() {
        let help = button_width("F1", " Help ".to_string());
        let search = button_width("F3", " Search:服务 ".to_string());
        let column = help.saturating_add(search).saturating_add(1);
        let action = footer_action_at(column, SortMode::Cpu, false, false, "", "服务", 1000, None);
        assert_eq!(action, Some(FooterAction::Filter));
    }
}
