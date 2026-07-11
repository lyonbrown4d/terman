use std::{collections::HashSet, 
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    app_tree_input::{apply_tree_branch, process_tree_active},
    app_events::{
        handle_filter_input, handle_help_input, handle_search_input,
        handle_signal_input, handle_sort_menu_input, selected_process_pid,
    },
    app_input::{
        adjust_refresh, apply_direct_sort, command_display_key, delay_key, environment_key, filter_key, follow_key, help_key, interrupt_key,
        invert_sort_key, kill_key, move_selection, navigation_key, next_tab, priority_delta,
        quit_key, search_key, sort_key, tree_branch_action, tree_key, tree_toggle_all_key,
        user_filter_key, TreeBranchAction,
    },
    app_mouse_action::{MouseActionResult, apply_mouse_action},
    command_display::ProcessCommandMode,
    environment_view::EnvironmentViewState,
    interrupt::InterruptFlag,
    metrics::Metrics,
    model::{IoRow, ProcessRow, SocketRow, SortMode},
    process_tree::ProcessTreeState,
    mouse::{self, MouseContext},
    render::Tab,
    setup_menu::SetupMenuState,
    signal_menu::SignalMenuState,
    user_filter::UserFilterState,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn poll_until_refresh(
    interrupt: &InterruptFlag,
    metrics: &mut Metrics,
    refresh_ms: &mut u64,
    tab: &mut Tab,
    sort: &mut SortMode,
    sort_inverted: &mut bool,
    sort_menu_open: &mut bool,
    sort_cursor: &mut SortMode,
    sort_header_pressed: &mut Option<SortMode>,
    setup_menu: &mut SetupMenuState,
    user_filter: &mut UserFilterState,
    tree: &mut bool,
    command_mode: &mut ProcessCommandMode,
    tree_state: &mut ProcessTreeState,
    help_open: &mut bool,
    signal_menu: &mut Option<SignalMenuState>,
    environment_view: &mut EnvironmentViewState,
    selected: &mut usize,
    followed_pid: &mut Option<String>,
    detail_scroll: &mut usize,
    io_scroll: &mut usize,
    network_scroll: &mut usize,
    processes: &[ProcessRow],
    process_users: &[String],
    io: &[IoRow],
    sockets: &[SocketRow],
    cpu_core_count: usize,
    filter: &mut String,
    filter_input: &mut Option<String>,
    search: &mut String,
    search_input: &mut Option<String>,
    tagged_pids: &mut HashSet<String>,
) -> io::Result<bool> {
    if interrupt.interrupted() {
        return Ok(true);
    }
    let refresh = Duration::from_millis((*refresh_ms).max(100));
    let deadline = Instant::now() + refresh;
    while Instant::now() < deadline {
        if interrupt.interrupted() {
            return Ok(true);
        }
        if event::poll(Duration::from_millis(50))? {
            let previous_selected = *selected;
            let redraw = match event::read()? {
                Event::Key(key) if interrupt_key(&key) => return Ok(true),
                Event::Key(key) if key.kind == KeyEventKind::Release => false,
                Event::Key(key) if environment_view.handle_key(key.code) => true,
                Event::Mouse(mouse) if environment_view.handle_mouse(mouse) => true,
                Event::Key(key) if key.code == KeyCode::F(10) => return Ok(true),
                Event::Key(key) if user_filter.handle_key(key.code) => true,
                Event::Mouse(mouse_event) if setup_menu.handle_mouse(mouse_event, refresh_ms, tree, sort_inverted) => true,
                Event::Mouse(mouse_event) => {
                    let action = mouse::handle_mouse(
                        mouse_event,
                        MouseContext {
                            tab,
                            sort,
                            sort_inverted,
                            sort_menu_open,
                            sort_cursor,
                            sort_header_pressed,
                            setup_menu,
                            user_filter,
                            tree,
                            help_open,
                            selected,
                            detail_scroll,
                            io_scroll,
                            network_scroll,
                            processes,
                            process_users,
                            io,
                            sockets,
                            cpu_core_count,
                            filter: filter_input.as_deref().unwrap_or(filter.as_str()),
                            search: search_input.as_deref().unwrap_or(search.as_str()),
                            signal_menu,
                            refresh_ms: *refresh_ms,
                        },
                    );
                    match apply_mouse_action(
                        action,
                        metrics,
                        signal_menu,
                        tagged_pids,
                        processes,
                        *selected,
                        refresh_ms,
                        tree_state,
                        search,
                        search_input,
                        filter,
                        filter_input,
                    ) {
                        MouseActionResult::Quit => return Ok(true),
                        MouseActionResult::Redraw => true,
                        MouseActionResult::Ignored => false,
                    }
                }
                Event::Key(key) if handle_signal_input(key.code, metrics, signal_menu) => true,
                Event::Key(key) if handle_help_input(key.code, help_open) => true,
                Event::Key(key)
                    if handle_sort_menu_input(
                        key.code,
                        sort,
                        sort_menu_open,
                        sort_cursor,
                    ) =>
                {
                    true
                }
                Event::Key(key)
                    if handle_search_input(
                        key.code,
                        search,
                        search_input,
                        selected,
                        processes,
                    ) =>
                {
                    true
                }
                Event::Key(key) if handle_filter_input(key.code, filter, filter_input) => true,
                Event::Key(key) if setup_menu.handle_key(key.code, refresh_ms, tree, sort_inverted) => true,
                Event::Key(key) if key.code == KeyCode::Esc && !filter.is_empty() => {
                    filter.clear();
                    true
                }
                Event::Key(key) if quit_key(key.code) => return Ok(true),
                Event::Key(key)
                    if crate::app_input::tag_key(key.code)
                        && matches!(*tab, Tab::Overview | Tab::Processes) =>
                {
                    crate::app_events::toggle_process_tag(
                        tagged_pids, processes, *selected,
                    );
                    true
                }
                Event::Key(key) if crate::app_input::untag_all_key(key.code) => {
                    tagged_pids.clear();
                    true
                }
                Event::Key(key)
                    if environment_key(key.code)
                        && matches!(*tab, Tab::Overview | Tab::Processes) =>
                {
                    if let Some(pid) =
                        selected_process_pid(processes, *selected)
                    {
                        let entries = metrics
                            .process_environment(&pid)
                            .unwrap_or_default();
                        environment_view.open(pid, entries);
                    }
                    true
                }
                Event::Key(key) if follow_key(key.code) => {
                    let pid = selected_process_pid(processes, *selected);
                    if followed_pid.as_deref() == pid.as_deref() {
                        *followed_pid = None;
                    } else {
                        *followed_pid = pid;
                    }
                    true
                }
                Event::Key(key) if navigation_key(key.code) => {
                    let next = move_selection(*selected, processes.len(), key.code);
                    if next != *selected {
                        *detail_scroll = 0;
                    }
                    *selected = next;
                    true
                }
                Event::Key(key)
                    if process_tree_active(*tab, *tree) && tree_toggle_all_key(key.code) =>
                {
                    tree_state.toggle_all();
                    true
                }
                Event::Key(key)
                    if process_tree_active(*tab, *tree)
                        && tree_branch_action(key.code).is_some() =>
                {
                    apply_tree_branch(
                        tree_state,
                        processes,
                        *selected,
                        tree_branch_action(key.code).unwrap_or(TreeBranchAction::Expand),
                    );
                    true
                }                Event::Key(key) if delay_key(key.code) => {
                    adjust_refresh(refresh_ms, key.code);
                    true
                }
                Event::Key(key) if kill_key(key.code) => {
                    *signal_menu = crate::app_events::signal_menu_for_processes(tagged_pids, processes, *selected);
                    true
                }
                Event::Key(key) if priority_delta(key.code).is_some() => {
                    crate::app_events::adjust_process_priorities(metrics, tagged_pids, processes, *selected, priority_delta(key.code).unwrap_or_default(),);
                    true
                }
                Event::Key(key) if help_key(key.code) => {
                    *help_open = true;
                    true
                }
                Event::Key(key) if search_key(key.code) => {
                    *search_input = Some(search.clone());
                    true
                }
                Event::Key(key) if filter_key(key.code) => {
                    *filter_input = Some(filter.clone());
                    true
                }
                Event::Key(key) if user_filter_key(key.code) => {
                    *sort_menu_open = false;
                    user_filter.open(process_users);
                    true
                }
                Event::Key(key) if apply_direct_sort(*tab, key.code, sort, sort_inverted) => true,
                Event::Key(key) if sort_key(key.code) => {
                    *sort_cursor = *sort;
                    *sort_menu_open = true;
                    true
                }
                Event::Key(key) if invert_sort_key(key.code) => {
                    *sort_inverted = !*sort_inverted;
                    true
                }
                Event::Key(key) if command_display_key(key.code) => { command_mode.toggle(); true }
                Event::Key(key) if tree_key(key.code) => {
                    *tree = !*tree;
                    true
                }
                Event::Key(key) => {
                    let next = next_tab(*tab, &key);
                    let changed = next != *tab;
                    *tab = next;
                    changed
                }
                Event::Resize(_, _) => true,
                _ => false,
            };
            if *selected != previous_selected {
                *followed_pid = None;
            }
            if redraw {
                return Ok(false);
            }
        }
    }
    Ok(false)
}
