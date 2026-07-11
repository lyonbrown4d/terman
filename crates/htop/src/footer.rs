use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::model::SortMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FooterAction {
    Help,
    Setup,
    User,
    Search,
    Filter,
    Tree,
    TreeExpand,
    TreeCollapse,
    TreeToggleAll,
    Sort,
    PriorityHigher,
    PriorityLower,
    Tag,
    UntagAll,
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
    user_filter: Option<&str>,
    filter: &str,
    filtering: bool,
    search: &str,
    searching: bool,
    refresh_ms: u64,
    kill_target: Option<&str>,
) -> Line<'static> {
    let mut spans = vec![
        key_span("F1"), value_span(format!(" {} ", terman_common::builtin_htop_footer_help_hint())),
        key_span("F2"), value_span(format!(" {} ", terman_common::builtin_htop_setup_title_hint())),
        key_span("u"), value_span(format!(" {}:{} ", terman_common::builtin_htop_user_filter_hint(), user_label(user_filter))),
        key_span("F3"), value_span(format!(" {}:{} ", terman_common::builtin_htop_footer_search_hint(), value_label(search))),
        key_span("F4"), value_span(format!(" {}:{} ", terman_common::builtin_htop_footer_filter_hint(), value_label(filter))),
        key_span("F5"), value_span(format!(" {} ", view_label(tree))),
        key_span("F6"), value_span(sort_label(sort, sort_inverted)),
        key_span("F7"), value_span(format!(" {} ", terman_common::builtin_htop_footer_priority_higher_hint())),
        key_span("F8"), value_span(format!(" {} ", terman_common::builtin_htop_footer_priority_lower_hint())),
        key_span("Spc"), value_span(format!(" {} ", terman_common::builtin_htop_tag())),
        key_span("U"), value_span(format!(" {} ", terman_common::builtin_htop_untag_all())),
        key_span("F9"), value_span(format!(" {} ", terman_common::builtin_htop_footer_kill_hint())),
    ];
    if tree {
        spans.extend([
            key_span("-"),
            value_span(format!(" {} ", terman_common::builtin_htop_tree_collapse_hint())),
            key_span("+"),
            value_span(format!(" {} ", terman_common::builtin_htop_tree_expand_hint())),
            key_span("*"),
            value_span(format!(" {} ", terman_common::builtin_htop_tree_toggle_all_hint())),
        ]);
    } else {
        spans.extend([
            key_span("+/-"),
            value_span(format!(" {}:{}ms ", terman_common::builtin_htop_footer_delay_hint(), refresh_ms)),
        ]);
    }
    spans.extend([key_span("F10"), value_span(format!(" {} ", terman_common::builtin_htop_footer_quit_hint()))]);
    spans.extend(prompt_spans(tree, filtering, searching, kill_target));
    Line::from(spans)
}

pub(crate) fn footer_action_at(
    column: u16,
    sort: SortMode,
    sort_inverted: bool,
    tree: bool,
    user_filter: Option<&str>,
    filter: &str,
    search: &str,
    refresh_ms: u64,
    kill_target: Option<&str>,
) -> Option<FooterAction> {
    let mut segments = vec![
        (FooterAction::Help, button_width("F1", format!(" {} ", terman_common::builtin_htop_footer_help_hint()))),
        (FooterAction::Setup, button_width("F2", format!(" {} ", terman_common::builtin_htop_setup_title_hint()))),
        (FooterAction::User, button_width("u", format!(" {}:{} ", terman_common::builtin_htop_user_filter_hint(), user_label(user_filter)))),
        (FooterAction::Search, button_width("F3", format!(" {}:{} ", terman_common::builtin_htop_footer_search_hint(), value_label(search)))),
        (FooterAction::Filter, button_width("F4", format!(" {}:{} ", terman_common::builtin_htop_footer_filter_hint(), value_label(filter)))),
        (FooterAction::Tree, button_width("F5", format!(" {} ", view_label(tree)))),
        (FooterAction::Sort, button_width("F6", sort_label(sort, sort_inverted))),
        (FooterAction::PriorityHigher, button_width("F7", format!(" {} ", terman_common::builtin_htop_footer_priority_higher_hint()))),
        (FooterAction::PriorityLower, button_width("F8", format!(" {} ", terman_common::builtin_htop_footer_priority_lower_hint()))),
        (FooterAction::Tag, button_width("Spc", format!(" {} ", terman_common::builtin_htop_tag()))),
        (FooterAction::UntagAll, button_width("U", format!(" {} ", terman_common::builtin_htop_untag_all()))),
        (FooterAction::Kill, button_width("F9", format!(" {} ", terman_common::builtin_htop_footer_kill_hint()))),
    ];
    if tree {
        segments.extend([
            (
                FooterAction::TreeCollapse,
                button_width("-", format!(" {} ", terman_common::builtin_htop_tree_collapse_hint())),
            ),
            (
                FooterAction::TreeExpand,
                button_width("+", format!(" {} ", terman_common::builtin_htop_tree_expand_hint())),
            ),
            (
                FooterAction::TreeToggleAll,
                button_width("*", format!(" {} ", terman_common::builtin_htop_tree_toggle_all_hint())),
            ),
        ]);
    } else {
        segments.push((
            FooterAction::DelayFaster,
            button_width("+/-", format!(" {}:{}ms ", terman_common::builtin_htop_footer_delay_hint(), refresh_ms)),
        ));
    }
    segments.push((FooterAction::Quit, button_width("F10", format!(" {} ", terman_common::builtin_htop_footer_quit_hint()))));
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
    format!(" {}:{} {} ", terman_common::builtin_htop_sort_menu_title_hint(), sort.label(), terman_common::builtin_htop_setup_direction_hint(inverted))
}

fn prompt_spans(
    tree: bool,
    filtering: bool,
    searching: bool,
    kill_target: Option<&str>,
) -> Vec<Span<'static>> {
    let Some(pid) = kill_target else {
        return vec![Span::styled(
            prompt_text(tree, filtering, searching, None),
            Style::default().fg(Color::Gray),
        )];
    };
    vec![
        Span::styled(terman_common::builtin_htop_signal_footer_hint(pid), Style::default().fg(Color::Gray)),
        key_span("Y"), value_span(format!(" {} ", terman_common::builtin_htop_footer_yes_hint())),
        key_span("N"), value_span(format!(" {} ", terman_common::builtin_htop_footer_no_hint())),
    ]
}

fn kill_prompt_action_at(column: u16, pid: &str) -> Option<FooterAction> {
    let mut start = terman_common::terminal_text_width(&terman_common::builtin_htop_signal_footer_hint(pid));
    let yes = button_width("Y", format!(" {} ", terman_common::builtin_htop_footer_yes_hint()));
    if column >= start && column < start.saturating_add(yes) { return Some(FooterAction::ConfirmKill); }
    start = start.saturating_add(yes);
    let no = button_width("N", format!(" {} ", terman_common::builtin_htop_footer_no_hint()));
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

fn prompt_text(tree: bool, filtering: bool, searching: bool, kill_target: Option<&str>) -> String {
    if let Some(pid) = kill_target {
        format!("{} {}/{}", terman_common::builtin_htop_signal_footer_hint(pid), terman_common::builtin_htop_footer_yes_hint(), terman_common::builtin_htop_footer_no_hint())
    } else if searching {
        format!(" {}", terman_common::builtin_htop_footer_search_prompt_hint())
    } else if filtering {
        format!(" {}", terman_common::builtin_htop_footer_filter_prompt_hint())
    } else if tree {
        format!(" {}", terman_common::builtin_htop_footer_tree_prompt_hint())
    } else {
        format!(" {}", terman_common::builtin_htop_footer_select_prompt_hint())
    }
}

fn user_label(user: Option<&str>) -> String {
    user.map(str::to_string)
        .unwrap_or_else(terman_common::builtin_htop_all_users_hint)
}

fn view_label(tree: bool) -> String {
    if tree {
        terman_common::builtin_htop_setup_tree_hint()
    } else {
        terman_common::builtin_htop_footer_view_flat_hint()
    }
}
#[cfg(test)]
mod tests {
    use super::{FooterAction, button_width, footer_action_at};
    use crate::model::SortMode;

    #[test]
    fn maps_footer_actions_after_wide_search_text() {
        let help = button_width("F1", format!(" {} ", terman_common::builtin_htop_footer_help_hint()));
        let setup = button_width("F2", format!(" {} ", terman_common::builtin_htop_setup_title_hint()));
        let user = button_width("u", format!(" {}:- ", terman_common::builtin_htop_user_filter_hint()));
        let search = button_width("F3", format!(" {}:服务 ", terman_common::builtin_htop_footer_search_hint()));
        let column = help.saturating_add(setup).saturating_add(user).saturating_add(search).saturating_add(1);
        let action = footer_action_at(column, SortMode::Cpu, false, false, None, "", "服务", 1000, None);
        assert_eq!(action, Some(FooterAction::Filter));
    }
}
