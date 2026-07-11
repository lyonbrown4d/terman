use std::collections::HashSet;

use crossterm::event::KeyCode;

use crate::{
    app_events::{
        adjust_process_priorities, confirm_mouse_signal,
        signal_menu_for_processes, toggle_process_tag,
    },
    app_input::{TreeBranchAction, adjust_refresh},
    app_tree_input::apply_tree_branch,
    metrics::Metrics,
    model::ProcessRow,
    mouse::MouseAction,
    process_tree::ProcessTreeState,
    signal_menu::SignalMenuState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseActionResult {
    Quit,
    Redraw,
    Ignored,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_mouse_action(
    action: MouseAction,
    metrics: &mut Metrics,
    signal_menu: &mut Option<SignalMenuState>,
    tagged_pids: &mut HashSet<String>,
    processes: &[ProcessRow],
    selected: usize,
    refresh_ms: &mut u64,
    tree_state: &mut ProcessTreeState,
    search: &str,
    search_input: &mut Option<String>,
    filter: &str,
    filter_input: &mut Option<String>,
) -> MouseActionResult {
    match action {
        MouseAction::Quit => return MouseActionResult::Quit,
        MouseAction::Search => {
            *search_input = Some(search.to_string());
        }
        MouseAction::Filter => {
            *filter_input = Some(filter.to_string());
        }
        MouseAction::Tag => {
            toggle_process_tag(tagged_pids, processes, selected);
        }
        MouseAction::UntagAll => tagged_pids.clear(),
        MouseAction::Kill => {
            *signal_menu =
                signal_menu_for_processes(tagged_pids, processes, selected);
        }
        MouseAction::ConfirmKill => {
            confirm_mouse_signal(metrics, signal_menu);
        }
        MouseAction::CancelKill => *signal_menu = None,
        MouseAction::PriorityHigher => {
            adjust_process_priorities(
                metrics,
                tagged_pids,
                processes,
                selected,
                -1,
            );
        }
        MouseAction::PriorityLower => {
            adjust_process_priorities(
                metrics,
                tagged_pids,
                processes,
                selected,
                1,
            );
        }
        MouseAction::DelayFaster => {
            adjust_refresh(refresh_ms, KeyCode::Char('+'));
        }
        MouseAction::DelaySlower => {
            adjust_refresh(refresh_ms, KeyCode::Char('-'));
        }
        MouseAction::TreeExpand => {
            apply_tree_branch(
                tree_state,
                processes,
                selected,
                TreeBranchAction::Expand,
            );
        }
        MouseAction::TreeCollapse => {
            apply_tree_branch(
                tree_state,
                processes,
                selected,
                TreeBranchAction::Collapse,
            );
        }
        MouseAction::TreeToggleAll => tree_state.toggle_all(),
        _ => {}
    }
    if action == MouseAction::Ignored {
        MouseActionResult::Ignored
    } else {
        MouseActionResult::Redraw
    }
}